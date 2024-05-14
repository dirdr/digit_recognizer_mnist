use log::error;
use ndarray::{Array, Array2};

use crate::initialization::InitializerType;

/// The `Layer` trait need to be implemented by any nn layer
/// a layer is defined as input nodes x and output nodes y
/// feed forward calculate the output nodes y with respect to the input nodes x
/// `propagate_backward` is responsible for two things :
/// 1: update trainable parameters (if any) in the layer (weights, bias)
/// 2: return the input gradient with respect to the output gradient,
/// the second role is used to propagate partial derivative backward in the network.
/// the layer inputs have shape (i, 1), layer outputs have shape (j, 1).
pub trait Layer {
    fn feed_forward(&mut self, input: &Array2<f64>) -> Array2<f64>;

    /// Return the input gradient vector (shape (i, 1)).
    /// # Arguments
    /// * `input` - Vector of input (shape (i, 1))
    /// * 'output_gradient' - Output gradient vector shape (j, 1))
    fn propagate_backward(
        &mut self,
        output_gradient: &Array2<f64>,
        learning_rate: f64,
    ) -> Array2<f64>;
}

/// `Dense` Layer (ie: Fully connected layer)
/// weights matrice follow the convension output first
/// weights_ji connect output node y_j to input node x_i
/// bias b_i is for the calculation of output node y_i
pub struct DenseLayer {
    /// shape (j, i) matrice of weights, w_ji connect node j to node i.
    weights: Array2<f64>,
    /// shape (j, 1) vector of bias
    bias: Array2<f64>,
    /// input passed during the feed forward step
    input: Option<Array2<f64>>,
}

impl DenseLayer {
    /// Create a new `DenseLayer` filling it with random value. see `InitializerType` for
    /// initialization parameters
    fn new(input_size: usize, output_size: usize, init: InitializerType) -> Self {
        Self {
            weights: init.initialize(input_size, (output_size, input_size)),
            bias: init.initialize(input_size, (output_size, 1)),
            input: None,
        }
    }
}

impl Layer for DenseLayer {
    /// Calculate the output vector (shape (j, 1))
    /// and store the passed input inside the layer to be used in the backpropagation
    fn feed_forward(&mut self, input: &Array2<f64>) -> Array2<f64> {
        self.input = Some(input.clone());
        self.weights.dot(input) + &self.bias
    }

    /// Update trainable parameters (weights and bias)
    /// and return the input gradient vector (shape (i, 1)).
    /// # Arguments
    /// * `input` - (shape (i, 1))
    /// * `output_gradient` - (shape (j, 1))
    fn propagate_backward(
        &mut self,
        output_gradient: &Array2<f64>,
        learning_rate: f64,
    ) -> Array2<f64> {
        let input = self
            .input
            .unwrap_or_else(|| panic!("access to a unset input inside backproapgation"));
        let weights_gradient = output_gradient.dot(&input.t());
        let input_gradient = self.weights.t().dot(output_gradient);
        self.bias
            .scaled_add(-1.0, &output_gradient.mapv(|e| e * learning_rate));
        self.weights
            .scaled_add(-1.0, &weights_gradient.mapv(|e| e * learning_rate));
        input_gradient
    }
}

/// The `ActivationLayer` apply a activation function to it's input node to yield the output nodes.
struct ActivationLayer {
    pub activation_type: ActivationType,
    pub input: Option<Array2<f64>>,
}

impl Layer for ActivationLayer {
    /// apply the activation fonction to each input (shape (i * 1))
    /// return an output vector of shape (i * 1).
    fn feed_forward(&mut self, input: &Array2<f64>) -> Array2<f64> {
        self.activation_type.apply(input)
    }

    /// return the input gradient with respect to the activation layer output gradient
    /// because the activation layer doesn't have trainable parameters, we don't care about the
    /// learning_rate.
    fn propagate_backward(&mut self, output_gradient: &Array2<f64>, _: f64) -> Array2<f64> {
        let input = self
            .input
            .unwrap_or_else(|| panic!("access to a unset input inside backproapgation"));
        output_gradient * self.activation_type.derivative_apply(&input)
    }
}

/// The `SoftmaxLayer` is used just before the output to normalize probability of the logits.
/// This doesn't impl the `Layer` trait because we don't need to propagate the cost gradient
/// backward through this, reason is that this layer is used between the logit and the cost function to
/// normalize prediction probability, but we can easily calculate the gradient of the cost function with
/// respect to the logits and thus we don't need to propagate anything through this.
struct Softmax;

impl Softmax {
    /// Apply the softmax transformation to the input vector (shape (i, 1))
    /// return the probability distribution vector (shape (i, 1))
    fn transform(&self, input: &Array2<f64>) -> Array2<f64> {
        let max_logit = input.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let sum_exps = input.clone().mapv(|e| f64::exp(e - max_logit)).sum();
        input.mapv(|e| f64::exp(e - max_logit) / sum_exps)
    }
}

pub enum ActivationType {
    ReLU,
}

impl ActivationType {
    /// Apply an activation function to the given input
    fn apply(&self, input: &Array2<f64>) -> Array2<f64> {
        match self {
            Self::ReLU => input.mapv(|e| 0f64.max(e)),
        }
    }

    /// Apply the derivative
    fn derivative_apply(&self, input: &Array2<f64>) -> Array2<f64> {
        match self {
            Self::ReLU => input.mapv(|e| if e >= 0f64 { 1f64 } else { 0f64 }),
        }
    }
}
