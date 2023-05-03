use crate::{backend::Backend, BasicOps, Shape, Tensor};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::ops::Range;

/// The struct should always be used with the [check](crate::check) macro.
///
/// This is a simple public crate data structure that efficiently checks tensor operations and
/// formats clear error messages. It's crucial that the checks are really fast, but it doesn't matter
/// when a failed check is discovered since the program will panic.
///
/// # Notes
///
/// Failing tensor checks will always result in a panic.
/// As mentioned in [The Rust Programming Language book](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html),
/// when there is no way to recover, panic should be used instead of a result.
///
/// Most users will unwrap the results anyway, which will worsen the clarity of the code. Almost
/// all checks highlight programming errors, which means invalid programs that should be fixed.
/// Checks are not the ideal way to help users write correct programs, but they are still better
/// than backend errors. Other forms of compile-time validation could be developed, such as named
/// tensors, but we have to carefully evaluate the ease of use of the Tensor API. Adding overly
/// complex type validation checks might drastically worsen the API and result in harder-to-maintain
/// programs.
///
/// # Design
///
/// Maybe the Backend API should return a result for each operation, which would allow handling
/// all checks, even the ones that can't be efficiently checked before performing an operation,
/// such as the `index_select` operation. The downside of that approach is that all backend
/// implementation might re-implement the same checks, which may result in uncessary code
/// duplication. Maybe a combination of both strategies could help to cover all usecases.
pub enum TensorCheck {
    Ok,
    Failed(FailedTensorCheck),
}

impl TensorCheck {
    /// Checks device and shape compatibility for element wise binary operations.
    pub fn binary_ops_ew<B: Backend, const D: usize, K: BasicOps<B>>(
        ops: &str,
        lhs: &Tensor<B, D, K>,
        rhs: &Tensor<B, D, K>,
    ) -> Self {
        Self::Ok
            .binary_ops_device(ops, &lhs.device(), &rhs.device())
            .binary_ops_ew_shape(ops, &lhs.shape(), &rhs.shape())
    }

    pub fn into_scalar<const D: usize>(shape: &Shape<D>) -> Self {
        let mut check = Self::Ok;

        if shape.num_elements() != 1 {
            check = check.register(
                "Into Scalar",
                TensorError::new("Only tensors with 1 element can be converted into scalar.")
                    .details(format!(
                        "Current tensor has {} elements",
                        shape.num_elements()
                    )),
            );
        }

        check
    }
    pub fn permute<const D: usize>(shape: &Shape<D>, permutation: [usize; D]) -> Self {
        Self::Ok
    }
    pub fn reshape<const D1: usize, const D2: usize>(
        original: &Shape<D1>,
        target: &Shape<D2>,
    ) -> Self {
        let mut check = Self::Ok;

        if original.num_elements() != target.num_elements() {
            check = check.register("Reshape", TensorError::new(
                "The given shape doesn't have the same number of elements as the current tensor.",
            )
            .details(format!(
                "Current shape: {:?}, target shape: {:?}.",
                original.dims, target.dims
            )));
        }

        check
    }

    pub fn flatten<const D1: usize, const D2: usize>(start_dim: usize, end_dim: usize) -> Self {
        let mut check = Self::Ok;

        if start_dim > end_dim {
            check = check.register(
                "Flatten",
                TensorError::new(format!(
                    "The start dim ({start_dim}) must be smaller than the end dim ({end_dim})"
                )),
            );
        }

        if D2 > D1 {
            check = check.register(
                "Flatten",
                TensorError::new(format!("Result dim ({D2}) must be smaller than ({D1})")),
            );
        }

        if D1 < end_dim + 1 {
            check = check.register(
                "Flatten",
                TensorError::new(format!(
                    "The end dim ({end_dim}) must be greater than the tensor dim ({D2})"
                )),
            );
        }

        check
    }

    pub fn unsqueeze<const D1: usize, const D2: usize>() -> Self {
        let mut check = Self::Ok;
        if D2 < D1 {
            check = check.register(
                "Unsqueeze",
                TensorError::new(format!(
                    "Can't unsqueeze smaller tensor, got dim {D2}, expected > {D1}"
                )),
            );
        }

        check
    }

    pub fn swap_dims<const D: usize>(dim1: usize, dim2: usize) -> Self {
        let mut check = Self::Ok;

        if dim1 > D || dim2 > D {
            check = check.register(
                "Swap Dims",
                TensorError::new("The swap dimensions must be smaller than the tensor dimension")
                    .details(format!(
                        "Swap dims ({dim1}, {dim2}) on tensor with ({D}) dimensions."
                    )),
            );
        }

        check
    }

    pub fn matmul<B: Backend, const D: usize>(lhs: &Tensor<B, D>, rhs: &Tensor<B, D>) -> Self {
        let mut check = Self::Ok;

        check = check.binary_ops_device("Matmul", &lhs.device(), &rhs.device());

        if D < 2 {
            return check;
        }

        let shape_lhs = lhs.shape();
        let shape_rhs = rhs.shape();

        let dim_lhs = shape_lhs.dims[D - 1];
        let dim_rhs = shape_rhs.dims[D - 2];

        if dim_lhs != dim_rhs {
            check = check.register(
                "Matmul",
                TensorError::new(format!(
                    "The inner dimension of matmul should be the same, but got {dim_lhs} and {dim_rhs}."
                ))
                .details(format!(
                    "Lhs shape {:?}, rhs shape {:?}.",
                    shape_lhs.dims, shape_rhs.dims
                )),
            );
        }

        check
    }

    pub fn cat<B: Backend, const D: usize, K: BasicOps<B>>(
        tensors: &[Tensor<B, D, K>],
        dim: usize,
    ) -> Self {
        let mut check = Self::Ok;

        if dim >= D {
            check = check.register(
                "Cat",
                TensorError::new(
                    "Can't concatenate tensors on a dim that exceeds the tensors dimension",
                )
                .details(format!(
                    "Trying to concatenate tensors with {D} dimensions on axis {dim}."
                )),
            );
        }

        if tensors.is_empty() {
            return check.register(
                "Cat",
                TensorError::new("Can't concatenate an empty list of tensors."),
            );
        }

        let mut shape_reference = tensors.get(0).unwrap().shape();
        shape_reference.dims[dim] = 1; // We want to check every dims except the one where the
                                       // concatenation happens.

        for tensor in tensors {
            let mut shape = tensor.shape();
            shape.dims[dim] = 1; // Ignore the concatenate dim.

            if shape_reference != shape {
                return check.register(
                    "Cat",
                    TensorError::new("Can't concatenate tensors with different shapes, except for the provided dimension").details(
                        format!(
                            "Provided dimension ({}), tensors shapes: {:?}",
                            dim,
                            tensors.iter().map(Tensor::shape).collect::<Vec<_>>()
                        ),
                    ),
                );
            }
        }

        check
    }

    pub fn index<const D1: usize, const D2: usize>(
        shape: &Shape<D1>,
        indexes: &[Range<usize>; D2],
    ) -> Self {
        let mut check = Self::Ok;
        let n_dims_tensor = D1;
        let n_dims_indexes = D2;

        if n_dims_tensor < n_dims_indexes {
            check = check.register("Index", 
                TensorError::new ("The provided indexes array has a higher number of dimensions than the current tensor.")
                .details(
                    format!(
                    "The indexes array must be smaller or equal to the tensor number of dimensions. \
                    Tensor number of dimensions: {n_dims_tensor}, indexes array lenght {n_dims_indexes}."
                )));
        }

        for i in 0..usize::min(D1, D2) {
            let d_tensor = shape.dims[i];
            let index = indexes.get(i).unwrap();

            if index.end > d_tensor {
                check = check.register(
                    "Index",
                    TensorError::new("The provided indexes array has a range that exceeds the current tensor size.")
                    .details(format!(
                        "The range ({}..{}) exceeds the size of the tensor ({}) at dimension {}. \
                        Tensor shape {:?}, provided indexes {:?}.",
                        index.start,
                        index.end,
                        d_tensor,
                        i,
                        shape.dims,
                        indexes,
                    )));
            }

            if index.start >= index.end {
                check = check.register(
                    "Index",
                    TensorError::new("The provided indexes array has a range where the start index is bigger or equal to its end.")
                    .details(format!(
                        "The range at dimension '{}' starts at '{}' and is greater or equal to its end '{}'. \
                        Tensor shape {:?}, provided indexes {:?}.",
                        i,
                        index.start,
                        index.end,
                        shape.dims,
                        indexes,
                    )));
            }
        }

        check
    }

    pub fn index_assign<const D1: usize, const D2: usize>(
        shape: &Shape<D1>,
        shape_value: &Shape<D1>,
        indexes: &[Range<usize>; D2],
    ) -> Self {
        let mut check = Self::Ok;

        if D1 < D2 {
            check = check.register("Index Assign",
                TensorError::new ("The provided indexes array has a higher number of dimensions than the current tensor.")
                .details(
                    format!(
                    "The indexes array must be smaller or equal to the tensor number of dimensions. \
                    Tensor number of dimensions: {D1}, indexes array lenght {D2}."
                )));
        }

        for i in 0..usize::min(D1, D2) {
            let d_tensor = shape.dims[i];
            let d_tensor_value = shape_value.dims[i];
            let index = indexes.get(i).unwrap();

            if index.end > d_tensor {
                check = check.register(
                    "Index Assign",
                    TensorError::new("The provided indexes array has a range that exceeds the current tensor size.")
                    .details(format!(
                        "The range ({}..{}) exceeds the size of the tensor ({}) at dimension {}. \
                        Current tensor shape {:?}, value tensor shape {:?}, provided indexes {:?}.",
                        index.start,
                        index.end,
                        d_tensor,
                        i,
                        shape.dims,
                        shape_value.dims,
                        indexes,
                    )));
            }

            if index.end - index.start != d_tensor_value {
                check = check.register(
                    "Index Assign",
                    TensorError::new("The value tensor must match the amount of elements selected with the indexes array")
                    .details(format!(
                        "The range ({}..{}) doesn't match the number of elements of the value tensor ({}) at dimension {}. \
                        Current tensor shape {:?}, value tensor shape {:?}, provided indexes {:?}.",
                        index.start,
                        index.end,
                        d_tensor_value,
                        i,
                        shape.dims,
                        shape_value.dims,
                        indexes,
                    )));
            }

            if index.start >= index.end {
                check = check.register(
                    "Index Assign",
                    TensorError::new("The provided indexes array has a range where the start index is bigger or equal to its end.")
                    .details(format!(
                        "The range at dimension '{}' starts at '{}' and is greater or equal to its end '{}'. \
                        Current tensor shape {:?}, value tensor shape {:?}, provided indexes {:?}.",
                        i,
                        index.start,
                        index.end,
                        shape.dims,
                        shape_value.dims,
                        indexes,
                    )));
            }
        }

        check
    }

    /// Checks aggregate dimension such as mean and sum.
    pub fn aggregate_dim<const D: usize>(ops: &str, dim: usize) -> Self {
        let mut check = Self::Ok;

        if dim > D {
            check = check.register(
                ops,
                TensorError::new(format!(
                    "Can't aggregate a tensor with ({D}) dimensions on axis ({dim})"
                )),
            );
        }

        check
    }

    /// The goal is to minimize the cost of checks when there are no error, but it's way less
    /// important when an error occured, crafting a comprehensive error message is more important
    /// than optimizing string manipulation.
    fn register(self, ops: &str, error: TensorError) -> Self {
        let errors = match self {
            Self::Ok => vec![error],
            Self::Failed(mut failed) => {
                failed.errors.push(error);
                failed.errors
            }
        };

        Self::Failed(FailedTensorCheck {
            ops: ops.to_string(),
            errors,
        })
    }

    /// Checks if shapes are compatible for element wise operations supporting broadcasting.
    pub fn binary_ops_ew_shape<const D: usize>(
        self,
        ops: &str,
        lhs: &Shape<D>,
        rhs: &Shape<D>,
    ) -> Self {
        let mut check = self;

        for i in 0..D {
            let d_lhs = lhs.dims[i];
            let d_rhs = rhs.dims[i];

            if d_lhs != d_rhs {
                let is_broadcast = d_lhs == 1 || d_rhs == 1;

                if is_broadcast {
                    continue;
                }

                check = check.register(ops,
                    TensorError::new("The provided tensors have incompatible shapes.")
                    .details(format!(
                    "Incompatible size at dimension '{}' => '{} != {}', which can't be broadcasted. \
                    Lhs tensor shape {:?}, Rhs tensor shape {:?}.",
                    i,
                    d_lhs,
                    d_rhs,
                    lhs.dims,
                    rhs.dims,
                 )));
            }
        }

        check
    }

    /// Checks if tensor devices are equal.
    fn binary_ops_device<Device: PartialEq + core::fmt::Debug>(
        self,
        ops: &str,
        lhs: &Device,
        rhs: &Device,
    ) -> Self {
        match lhs != rhs {
            true => self.register(
                ops,
                TensorError::new("The provided tensors are not on the same device.").details(
                    format!("Lhs tensor device {lhs:?}, Rhs tensor device {rhs:?}.",),
                ),
            ),
            false => self,
        }
    }
}

pub struct FailedTensorCheck {
    ops: String,
    errors: Vec<TensorError>,
}

impl FailedTensorCheck {
    /// Format all the checks into a single message ready to be printed by a [panic](core::panic).
    pub fn format(self) -> String {
        self.errors.into_iter().enumerate().fold(
            format!(
                "=== Tensor Operation Error ===\n  Operation: '{}'\n  Reason:",
                self.ops
            ),
            |accum, (number, error)| accum + error.format(number + 1).as_str(),
        ) + "\n"
    }
}

struct TensorError {
    description: String,
    details: Option<String>,
}

impl TensorError {
    pub fn new<S: Into<String>>(description: S) -> Self {
        TensorError {
            description: description.into(),
            details: None,
        }
    }

    pub fn details<S: Into<String>>(mut self, details: S) -> Self {
        self.details = Some(details.into());
        self
    }

    fn format(self, number: usize) -> String {
        let mut message = format!("\n    {number}. ");
        message += self.description.as_str();
        message += " ";

        if let Some(details) = self.details {
            message += details.as_str();
            message += " ";
        }

        message
    }
}

/// We use a macro for all checks, since the panic message file and line number will match the
/// function that does the check instead of a the generic error.rs crate private unreleated file
/// and line number.
#[macro_export(local_inner_macros)]
macro_rules! check {
    ($check:expr) => {
        if let TensorCheck::Failed(check) = $check {
            core::panic!("{}", check.format());
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn reshape_invalid_shape() {
        check!(TensorCheck::reshape(
            &Shape::new([2, 2]),
            &Shape::new([1, 3])
        ));
    }

    #[test]
    fn reshape_valid_shape() {
        check!(TensorCheck::reshape(
            &Shape::new([2, 2]),
            &Shape::new([1, 4])
        ));
    }

    #[test]
    #[should_panic]
    fn index_range_exceed_dimension() {
        check!(TensorCheck::index(
            &Shape::new([3, 5, 7]),
            &[0..2, 0..4, 1..8]
        ));
    }

    #[test]
    #[should_panic]
    fn index_range_exceed_number_of_dimensions() {
        check!(TensorCheck::index(&Shape::new([3, 5]), &[0..1, 0..1, 0..1]));
    }

    #[test]
    #[should_panic]
    fn binary_ops_shapes_no_broadcast() {
        check!(TensorCheck::binary_ops_ew_shape(
            TensorCheck::Ok,
            "TestOps",
            &Shape::new([3, 5]),
            &Shape::new([3, 6])
        ));
    }

    #[test]
    fn binary_ops_shapes_with_broadcast() {
        check!(TensorCheck::binary_ops_ew_shape(
            TensorCheck::Ok,
            "Test",
            &Shape::new([3, 5]),
            &Shape::new([1, 5])
        ));
    }

    #[test]
    #[should_panic]
    fn binary_ops_devices() {
        check!(TensorCheck::binary_ops_device(
            TensorCheck::Ok,
            "Test",
            &5, // We can pass anything that implements PartialEq as device
            &8
        ));
    }
}
