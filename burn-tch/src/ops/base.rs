use burn_tensor::Shape;
use tch::Scalar;

use crate::{TchShape, TchTensor};
use std::{marker::PhantomData, ops::Range};

pub struct TchOps<E: tch::kind::Element + Copy + Default> {
    e: PhantomData<E>,
}

impl<E: tch::kind::Element + Copy + Default> TchOps<E> {
    pub fn reshape<const D1: usize, const D2: usize>(
        tensor: TchTensor<E, D1>,
        shape: Shape<D2>,
    ) -> TchTensor<E, D2> {
        let shape_tch: TchShape<D2> = shape.into();

        TchTensor::from_existing(tensor.tensor.reshape(shape_tch.dims), tensor.storage)
    }

    pub fn index<const D1: usize, const D2: usize>(
        tensor: TchTensor<E, D1>,
        indexes: [Range<usize>; D2],
    ) -> TchTensor<E, D1> {
        let storage = tensor.storage.clone();
        let mut tensor = tensor.tensor.shallow_clone();

        for (i, index) in indexes.iter().enumerate().take(D2) {
            let start = index.start as i64;
            let length = (index.end - index.start) as i64;
            tensor = tensor.narrow(i as i64, start, length);
        }

        TchTensor::from_existing(tensor, storage)
    }

    pub fn index_assign<const D1: usize, const D2: usize>(
        tensor: TchTensor<E, D1>,
        indexes: [Range<usize>; D2],
        value: TchTensor<E, D1>,
    ) -> TchTensor<E, D1> {
        let tensor_original = tensor.tensor.copy();
        let tch_shape = TchShape::from(tensor.shape());

        let mut tensor = tensor_original.view_(tch_shape.dims);

        for (i, index) in indexes.into_iter().enumerate().take(D2) {
            let start = index.start as i64;
            let length = (index.end - index.start) as i64;

            tensor = tensor.narrow(i as i64, start, length);
        }

        tensor.copy_(&value.tensor);

        TchTensor::new(tensor_original)
    }

    pub fn index_select<const D: usize>(
        tensor: TchTensor<E, D>,
        indexes: TchTensor<i64, D>,
    ) -> TchTensor<E, D> {
        let storage = tensor.storage.clone();
        let tensor = tensor.tensor.gather((D - 1) as i64, &indexes.tensor, false);

        TchTensor::from_existing(tensor, storage)
    }

    pub fn index_select_assign<const D: usize>(
        tensor: TchTensor<E, D>,
        indexes: TchTensor<i64, D>,
        value: TchTensor<E, D>,
    ) -> TchTensor<E, D> {
        let storage = tensor.storage.clone();
        let tensor = tensor
            .tensor
            .scatter_add((D - 1) as i64, &indexes.tensor, &value.tensor);

        TchTensor::from_existing(tensor, storage)
    }

    pub fn index_select_dim<const D: usize>(
        tensor: TchTensor<E, D>,
        dim: usize,
        indexes: TchTensor<i64, 1>,
    ) -> TchTensor<E, D> {
        let storage = tensor.storage.clone();
        let tensor = tensor.tensor.index_select(dim as i64, &indexes.tensor);

        TchTensor::from_existing(tensor, storage)
    }

    pub fn index_select_dim_assign<const D1: usize, const D2: usize>(
        tensor: TchTensor<E, D1>,
        dim: usize,
        indexes: TchTensor<i64, 1>,
        value: TchTensor<E, D2>,
    ) -> TchTensor<E, D1> {
        let mut indices = Vec::with_capacity(D1);
        for _ in 0..D1 {
            indices.push(None);
        }
        indices[dim] = Some(indexes.tensor);

        tensor.unary_ops(
            |mut tensor| tensor.index_put_(&indices, &value.tensor, true),
            |tensor| tensor.index_put(&indices, &value.tensor, true),
        )
    }

    pub fn cat<const D: usize>(tensors: Vec<TchTensor<E, D>>, dim: usize) -> TchTensor<E, D> {
        let tensors: Vec<tch::Tensor> = tensors
            .into_iter()
            .map(|t| t.tensor.shallow_clone())
            .collect();
        let tensor = tch::Tensor::cat(&tensors, dim as i64);

        TchTensor::new(tensor)
    }
    pub fn stack<const D: usize, const D2: usize>(
        tensors: Vec<TchTensor<E, D>>,
        dim: usize,
    ) -> TchTensor<E, D2> {
        let tensors: Vec<tch::Tensor> = tensors
            .into_iter()
            .map(|t| t.tensor.shallow_clone())
            .collect();
        let tensor = tch::Tensor::stack(&tensors, dim as i64);

        TchTensor::new(tensor)
    }
    

    pub fn unbind<const D: usize, const D2: usize>(
        tensor: TchTensor<E, D>,
        dim: usize,
    ) -> Vec<TchTensor<E, D2>> {
        let tensor = tensor.tensor.shallow_clone();
        let tensors = tensor.unbind(dim as i64);
        tensors.iter().map(|t| TchTensor::new(t.copy())).collect()
    }

    pub fn equal<const D: usize>(lhs: TchTensor<E, D>, rhs: TchTensor<E, D>) -> TchTensor<bool, D> {
        TchTensor::binary_ops_tensor(
            lhs,
            rhs,
            |lhs, rhs| lhs.eq_tensor_(rhs).to_kind(tch::Kind::Bool),
            |lhs, rhs| rhs.eq_tensor_(lhs).to_kind(tch::Kind::Bool),
            |lhs, rhs| lhs.eq_tensor(rhs),
        )
    }

    pub fn equal_elem<const D: usize, S: Into<tch::Scalar> + Clone>(
        lhs: TchTensor<E, D>,
        rhs: S,
    ) -> TchTensor<bool, D> {
        lhs.unary_ops(
            |mut tensor| tensor.eq_(rhs.clone().into()).to_kind(tch::Kind::Bool),
            |tensor| tensor.eq(rhs.clone().into()),
        )
    }

    pub fn greater<const D: usize>(
        lhs: TchTensor<E, D>,
        rhs: TchTensor<E, D>,
    ) -> TchTensor<bool, D> {
        TchTensor::binary_ops_tensor(
            lhs,
            rhs,
            |lhs, rhs| lhs.greater_tensor_(rhs).to_kind(tch::Kind::Bool),
            |lhs, rhs| rhs.less_tensor_(lhs).to_kind(tch::Kind::Bool),
            |lhs, rhs| lhs.greater_tensor(rhs),
        )
    }

    pub fn greater_elem<const D: usize, S: Into<tch::Scalar> + Clone>(
        lhs: TchTensor<E, D>,
        rhs: S,
    ) -> TchTensor<bool, D> {
        lhs.unary_ops(
            |mut tensor| tensor.greater_(rhs.clone().into()).to_kind(tch::Kind::Bool),
            |tensor| tensor.greater(rhs.clone().into()),
        )
    }

    pub fn greater_equal<const D: usize>(
        lhs: TchTensor<E, D>,
        rhs: TchTensor<E, D>,
    ) -> TchTensor<bool, D> {
        TchTensor::binary_ops_tensor(
            lhs,
            rhs,
            |lhs, rhs| lhs.greater_equal_tensor_(rhs).to_kind(tch::Kind::Bool),
            |lhs, rhs| rhs.less_equal_tensor_(lhs).to_kind(tch::Kind::Bool),
            |lhs, rhs| lhs.greater_equal_tensor(rhs),
        )
    }

    pub fn greater_equal_elem<const D: usize, S: Into<Scalar> + Clone>(
        lhs: TchTensor<E, D>,
        rhs: S,
    ) -> TchTensor<bool, D> {
        lhs.unary_ops(
            |mut tensor| {
                tensor
                    .greater_equal_(rhs.clone().into())
                    .to_kind(tch::Kind::Bool)
            },
            |tensor| tensor.greater_equal(rhs.clone().into()),
        )
    }

    pub fn lower<const D: usize>(lhs: TchTensor<E, D>, rhs: TchTensor<E, D>) -> TchTensor<bool, D> {
        TchTensor::binary_ops_tensor(
            lhs,
            rhs,
            |lhs, rhs| lhs.less_tensor_(rhs).to_kind(tch::Kind::Bool),
            |lhs, rhs| rhs.greater_tensor_(lhs).to_kind(tch::Kind::Bool),
            |lhs, rhs| lhs.less_tensor(rhs),
        )
    }

    pub fn lower_elem<const D: usize, S: Into<Scalar> + Clone>(
        lhs: TchTensor<E, D>,
        rhs: S,
    ) -> TchTensor<bool, D> {
        lhs.unary_ops(
            |mut tensor| tensor.less_(rhs.clone().into()).to_kind(tch::Kind::Bool),
            |tensor| tensor.less(rhs.clone().into()),
        )
    }

    pub fn lower_equal<const D: usize>(
        lhs: TchTensor<E, D>,
        rhs: TchTensor<E, D>,
    ) -> TchTensor<bool, D> {
        TchTensor::binary_ops_tensor(
            lhs,
            rhs,
            |lhs, rhs| lhs.less_equal_tensor_(rhs).to_kind(tch::Kind::Bool),
            |lhs, rhs| rhs.greater_equal_tensor_(lhs).to_kind(tch::Kind::Bool),
            |lhs, rhs| lhs.less_equal_tensor(rhs),
        )
    }

    pub fn lower_equal_elem<const D: usize, S: Into<Scalar> + Clone>(
        lhs: TchTensor<E, D>,
        rhs: S,
    ) -> TchTensor<bool, D> {
        lhs.unary_ops(
            |mut tensor| {
                tensor
                    .less_equal_(rhs.clone().into())
                    .to_kind(tch::Kind::Bool)
            },
            |tensor| tensor.less_equal(rhs.clone().into()),
        )
    }

    pub fn add<const D: usize>(lhs: TchTensor<E, D>, rhs: TchTensor<E, D>) -> TchTensor<E, D> {
        TchTensor::binary_ops_tensor(
            lhs,
            rhs,
            |lhs, rhs| lhs.f_add_(rhs).unwrap(),
            |lhs, rhs| rhs.f_add_(lhs).unwrap(),
            |lhs, rhs| lhs.f_add(rhs).unwrap(),
        )
    }

    pub fn sub<const D: usize>(lhs: TchTensor<E, D>, rhs: TchTensor<E, D>) -> TchTensor<E, D> {
        TchTensor::binary_ops_tensor(
            lhs,
            rhs,
            |lhs, rhs| lhs.f_sub_(rhs).unwrap(),
            |lhs, rhs| lhs.f_sub(rhs).unwrap(),
            |lhs, rhs| lhs.f_sub(rhs).unwrap(),
        )
    }

    pub fn mul<const D: usize>(lhs: TchTensor<E, D>, rhs: TchTensor<E, D>) -> TchTensor<E, D> {
        TchTensor::binary_ops_tensor(
            lhs,
            rhs,
            |lhs, rhs| lhs.f_mul_(rhs).unwrap(),
            |lhs, rhs| rhs.f_mul_(lhs).unwrap(),
            |lhs, rhs| lhs.f_mul(rhs).unwrap(),
        )
    }

    pub fn div<const D: usize>(lhs: TchTensor<E, D>, rhs: TchTensor<E, D>) -> TchTensor<E, D> {
        TchTensor::binary_ops_tensor(
            lhs,
            rhs,
            |lhs, rhs| lhs.f_div_(rhs).unwrap(),
            |lhs, rhs| lhs.f_div(rhs).unwrap(),
            |lhs, rhs| lhs.f_div(rhs).unwrap(),
        )
    }

    pub fn mean<const D: usize>(tensor: TchTensor<E, D>) -> TchTensor<E, 1> {
        let tensor = tensor.tensor.mean(E::KIND);
        TchTensor::new(tensor)
    }

    pub fn sum<const D: usize>(tensor: TchTensor<E, D>) -> TchTensor<E, 1> {
        let tensor = tensor.tensor.sum(E::KIND);
        TchTensor::new(tensor)
    }

    pub fn mean_dim<const D: usize>(tensor: TchTensor<E, D>, dim: usize) -> TchTensor<E, D> {
        TchTensor::from_existing(
            tensor
                .tensor
                .mean_dim(Some([dim as i64].as_slice()), true, E::KIND),
            tensor.storage,
        )
    }

    pub fn sum_dim<const D: usize>(tensor: TchTensor<E, D>, dim: usize) -> TchTensor<E, D> {
        TchTensor::from_existing(
            tensor
                .tensor
                .sum_dim_intlist(Some([dim as i64].as_slice()), true, E::KIND),
            tensor.storage,
        )
    }

    pub fn argmax<const D: usize>(tensor: TchTensor<E, D>, dim: usize) -> TchTensor<i64, D> {
        let storage = tensor.storage.clone();
        let tensor = tensor.tensor.argmax(dim as i64, true);

        TchTensor::from_existing(tensor, storage)
    }

    pub fn argmin<const D: usize>(tensor: TchTensor<E, D>, dim: usize) -> TchTensor<i64, D> {
        let storage = tensor.storage.clone();
        let tensor = tensor.tensor.argmin(dim as i64, true);

        TchTensor::from_existing(tensor, storage)
    }

    pub fn max_dim<const D: usize>(tensor: TchTensor<E, D>, dim: usize) -> TchTensor<E, D> {
        let storage = tensor.storage.clone();
        let (tensor, _indexes) = tensor.tensor.max_dim(dim as i64, true);

        TchTensor::from_existing(tensor, storage)
    }

    pub fn max_dim_with_indexes<const D: usize>(
        tensor: TchTensor<E, D>,
        dim: usize,
    ) -> (TchTensor<E, D>, TchTensor<i64, D>) {
        let storage = tensor.storage.clone();
        let (tensor, indexes) = tensor.tensor.max_dim(dim as i64, true);

        let tensor = TchTensor::from_existing(tensor, storage);
        let indexes = TchTensor::new(indexes);

        (tensor, indexes)
    }

    pub fn min_dim<const D: usize>(tensor: TchTensor<E, D>, dim: usize) -> TchTensor<E, D> {
        let storage = tensor.storage.clone();
        let (tensor, _indexes) = tensor.tensor.min_dim(dim as i64, true);

        TchTensor::from_existing(tensor, storage)
    }

    pub fn min_dim_with_indexes<const D: usize>(
        tensor: TchTensor<E, D>,
        dim: usize,
    ) -> (TchTensor<E, D>, TchTensor<i64, D>) {
        let storage = tensor.storage.clone();
        let (tensor, indexes) = tensor.tensor.min_dim(dim as i64, true);

        let tensor = TchTensor::from_existing(tensor, storage);
        let indexes = TchTensor::new(indexes);

        (tensor, indexes)
    }
}
