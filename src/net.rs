use burn::{
    Tensor,
    config::Config,
    module::Module,
    nn::{
        Linear, LinearConfig, PaddingConfig2d, Relu,
        conv::{Conv2d, Conv2dConfig},
    },
    tensor::{
        TensorData,
        activation::{softmax, tanh},
        backend::Backend,
    },
};

use crate::{
    constants::convert_global_coordinates,
    game::{BoardState, MainBoard},
};

#[derive(Module, Debug)]
struct NeuralNet<B: Backend> {
    conv1: Conv2d<B>,
    conv2: Conv2d<B>,
    policy_conv: Conv2d<B>,
    policy_fc: Linear<B>,
    value_conv: Conv2d<B>,
    value_fc1: Linear<B>,
    value_fc2: Linear<B>,
    activation: Relu,
}

#[derive(Config, Debug)]
struct NeuralNetConfig {
    #[config(default = 5)]
    input_channels: usize,
    #[config(default = 64)]
    conv1_channels: usize,
    #[config(default = 128)]
    conv2_channels: usize,
    #[config(default = 81)]
    num_cells: usize,
    #[config(default = 64)]
    policy_units: usize,
}

// 5x81 architecture (sequence of local boards, 5x9x3x3)
// channel 1: X placements
// channel 2: O placements
// channel 3: X wins (local board)
// channel 4: O wins (local board)
// channel 5: board to move
struct InputLayer([f32; 405]);

impl NeuralNetConfig {
    pub fn init<B: Backend>(&self, device: &B::Device) -> NeuralNet<B> {
        let conv1 = Conv2dConfig::new([self.input_channels, self.conv1_channels], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .init(device);
        let conv2 = Conv2dConfig::new([self.conv1_channels, self.conv2_channels], [3, 3])
            .with_padding(PaddingConfig2d::Same)
            .init(device);
        let policy_conv = Conv2dConfig::new([self.conv2_channels, 2], [1, 1]).init(device);
        let policy_fc = LinearConfig::new(2 * self.num_cells, self.num_cells).init(device);
        let value_conv = Conv2dConfig::new([self.conv2_channels, 1], [1, 1]).init(device);
        let value_fc1 = LinearConfig::new(self.num_cells, self.policy_units).init(device);
        let value_fc2 = LinearConfig::new(self.policy_units, 1).init(device);
        NeuralNet {
            conv1,
            conv2,
            policy_conv,
            policy_fc,
            value_conv,
            value_fc1,
            value_fc2,
            activation: Relu::new(),
        }
    }
}

impl<B: Backend> NeuralNet<B> {
    pub fn forward(&self, x: Tensor<B, 4>) -> (Tensor<B, 2>, Tensor<B, 2>) {
        let x = self.activation.forward(self.conv1.forward(x));
        let x = self.activation.forward(self.conv2.forward(x));

        let p = self.activation.forward(self.policy_conv.forward(x.clone()));
        let p = p.flatten(1, 3);
        let p = self.policy_fc.forward(p);

        let v = self.activation.forward(self.value_conv.forward(x));
        let v = v.flatten(1, 3);
        let v = self.activation.forward(self.value_fc1.forward(v));
        let v = tanh(self.value_fc2.forward(v));

        (p, v)
    }

    pub fn process_policy(
        p: Tensor<B, 2>,
        legal_moves_array: [f32; 81],
        device: &B::Device,
    ) -> Tensor<B, 2> {
        let legal_moves_tensor =
            Tensor::from_data(TensorData::new(legal_moves_array.to_vec(), [1, 81]), device);
        let processed_p = p.add(legal_moves_tensor);
        softmax(processed_p, 1)
    }

    pub fn get_legal_moves_array(main_board: MainBoard) -> [f32; 81] {
        let mut array = [f32::NEG_INFINITY; 81];
        if let Some(board_to_move) = main_board.board_to_move {
            let empty_mask = main_board.local_boards[board_to_move as usize].get_empty();
            for local_cell in empty_mask {
                let index = convert_global_coordinates(board_to_move, local_cell) as usize;
                array[index] = 0.0;
            }
        } else {
            for main_cell in 0..9 {
                let local_board = main_board.local_boards[main_cell as usize];
                if local_board.state != BoardState::InPlay {
                    continue;
                }
                let empty_mask = local_board.get_empty();
                for local_cell in empty_mask {
                    let index = convert_global_coordinates(main_cell, local_cell) as usize;
                    array[index] = 0.0;
                }
            }
        }

        array
    }
    pub fn get_input_layer(main_board: MainBoard) -> InputLayer {
        let mut input_layer = [0.0; 405];
        for (main_cell, local_board) in main_board.local_boards.iter().enumerate() {
            for x_cell in local_board.x {
                let index = convert_global_coordinates(main_cell as u8, x_cell) as usize;
                input_layer[index] = 1.0;
            }

            for o_cell in local_board.o {
                let index = 81 + convert_global_coordinates(main_cell as u8, o_cell) as usize;
                input_layer[index] = 1.0;
            }
        }

        for x_board in main_board.main_board.x {
            for i in 0..9 {
                let index = i + 81 * 2 + convert_global_coordinates(x_board, 0) as usize;
                input_layer[index] = 1.0;
            }
        }

        for o_board in main_board.main_board.o {
            for i in 0..9 {
                let index = i + 81 * 3 + convert_global_coordinates(o_board, 0) as usize;
                input_layer[index] = 1.0;
            }
        }

        if let Some(board_to_move) = main_board.board_to_move {
            for i in 0..9 {
                let index = i + 81 * 4 + convert_global_coordinates(board_to_move, 0) as usize;
                input_layer[index] = 1.0;
            }
        } else {
            input_layer[81 * 4..81 * 5].fill(1.0);
        }

        InputLayer(input_layer)
    }
}
