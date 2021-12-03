use halo2::{
    circuit::Chip,
    plonk::{Advice, Column, ConstraintSystem, Expression, VirtualCells},
    poly::Rotation,
};
use pasta_curves::arithmetic::FieldExt;
use std::marker::PhantomData;

use crate::param::HASH_WIDTH;

#[derive(Clone, Debug)]
pub(crate) struct AccountLeafStorageCodehashConfig {}

// Verifies the hash of a leaf is in the parent branch.
pub(crate) struct AccountLeafStorageCodehashChip<F> {
    config: AccountLeafStorageCodehashConfig,
    _marker: PhantomData<F>,
}

impl<F: FieldExt> AccountLeafStorageCodehashChip<F> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        q_enable: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>,
        s_rlp2: Column<Advice>,
        c_rlp2: Column<Advice>,
        s_advices: [Column<Advice>; HASH_WIDTH],
        c_advices: [Column<Advice>; HASH_WIDTH],
        acc_r: F,
        acc: Column<Advice>,
        acc_mult: Column<Advice>,
    ) -> AccountLeafStorageCodehashConfig {
        let config = AccountLeafStorageCodehashConfig {};

        meta.create_gate("account leaf storage codehash", |meta| {
            let q_enable = q_enable(meta);
            let mut constraints = vec![];

            // TODO: RLP properties

            // We have storage length in s_rlp2 (which is 160 presenting 128 + 32).
            // We have storage hash in s_advices.
            // We have codehash length in c_rlp2 (which is 160 presenting 128 + 32).
            // We have codehash in c_advices.

            let acc_prev = meta.query_advice(acc, Rotation::prev());
            let acc_mult_prev = meta.query_advice(acc_mult, Rotation::prev());
            let mut curr_r = acc_mult_prev.clone();
            let mut expr = acc_prev.clone()
                + meta.query_advice(s_rlp2, Rotation::cur()) * curr_r.clone();
            curr_r = curr_r * acc_r;
            for col in s_advices.iter() {
                let s = meta.query_advice(*col, Rotation::cur());
                expr = expr + s * curr_r.clone();
                curr_r = curr_r * acc_r;
            }

            expr = expr
                + meta.query_advice(c_rlp2, Rotation::cur()) * curr_r.clone();
            curr_r = curr_r * acc_r;
            for col in c_advices.iter() {
                let c = meta.query_advice(*col, Rotation::cur());
                expr = expr + c * curr_r.clone();
                curr_r = curr_r * acc_r;
            }

            let acc = meta.query_advice(acc, Rotation::cur());
            constraints.push((
                "account leaf storage codehash acc",
                q_enable.clone() * (expr.clone() - acc),
            ));

            constraints
        });

        config
    }

    pub fn construct(config: AccountLeafStorageCodehashConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }
}

impl<F: FieldExt> Chip<F> for AccountLeafStorageCodehashChip<F> {
    type Config = AccountLeafStorageCodehashConfig;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}