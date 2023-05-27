use core::{cmp::Ordering, fmt::Debug};
use dataflow_jit::ir::literal::{
    RowLiteral, StreamCollection,
    StreamCollection::{Map, Set},
};
use dbsp::{
    algebra::{AddByRef, HasZero, MulByRef, NegByRef, ZRingValue, ZSet},
    trace::{cursor::Cursor, ord::OrdZSet, BatchReader},
    zset, DBData, DBWeight,
};
use md5;
use sqlvalue::*;
use std::collections::BTreeMap;

#[derive(Eq, PartialEq)]
pub enum SortOrder {
    NONE,
    ROW,
    VALUE,
}

fn compare<T>(left: &Vec<T>, right: &Vec<T>) -> Ordering
where
    T: Ord,
{
    let llen = left.len();
    let rlen = right.len();
    let min;
    if llen < rlen {
        min = llen;
    } else {
        min = rlen;
    }
    for i in 0..min {
        let cmp = left[i].cmp(&right[i]);
        if cmp != Ordering::Equal {
            return cmp;
        }
    }
    return llen.cmp(&rlen);
}

/// Convert a zset to a vector of SqlRow.
/// Elements with > 1 weights will generate multiple SqlRows
/// # Panics
/// if any of the zset weights is negative
pub fn zset_to_rows<K, W>(set: &OrdZSet<K, W>) -> Vec<SqlRow>
where
    K: DBData + ToSqlRow,
    W: DBWeight + ZRingValue,
    usize: TryFrom<W>,
    <usize as TryFrom<W>>::Error: Debug,
{
    let mut result = Vec::with_capacity(set.weighted_count().try_into().unwrap());
    let mut cursor = set.cursor();
    while cursor.key_valid() {
        let mut w = cursor.weight();
        if !w.ge0() {
            panic!("Negative weight in output set!");
        }
        while !w.le0() {
            let row_vec = cursor.key().to_row();
            result.push(row_vec);
            w = w.add(W::neg(W::one()));
        }
        cursor.step_key();
    }
    result
}

/// Adds tuples of zsets.  Used to handle the outputs
/// of circuits that compute each a single view.
pub fn add_zset_tuple<K, W>(left: (OrdZSet<K, W>,), right: (OrdZSet<K, W>,)) -> (OrdZSet<K, W>,)
where
    K: DBData,
    W: DBWeight,
{
    (left.0.add_by_ref(&right.0),)
}

struct DataRows<'a> {
    rows: Vec<Vec<String>>,
    order: &'a SortOrder,
    format: &'a String,
}

impl<'a> DataRows<'a> {
    pub fn new(format: &'a String, order: &'a SortOrder) -> Self {
        Self {
            rows: Vec::new(),
            order: order,
            format: format,
        }
    }
    pub fn with_capacity(format: &'a String, order: &'a SortOrder, capacity: usize) -> Self {
        Self {
            rows: Vec::with_capacity(capacity),
            order: order,
            format: format,
        }
    }
    pub fn push(self: &mut Self, sql_row: SqlRow) {
        let row_vec = sql_row.to_slt_strings(&self.format);
        if *self.order == SortOrder::ROW || *self.order == SortOrder::NONE {
            self.rows.push(row_vec);
        } else if *self.order == SortOrder::VALUE {
            for r in row_vec {
                self.rows.push(vec![r])
            }
        }
    }

    pub fn get(mut self: Self) -> Vec<Vec<String>> {
        if *self.order != SortOrder::NONE {
            self.rows.sort_unstable_by(&compare);
        }
        self.rows
    }
}

/// The format is from the SqlLogicTest query output string format
pub fn zset_to_strings<K, W>(
    set: &OrdZSet<K, W>,
    format: String,
    order: SortOrder,
) -> Vec<Vec<String>>
where
    K: DBData + ToSqlRow,
    W: DBWeight + ZRingValue,
    usize: TryFrom<W>,
    <usize as TryFrom<W>>::Error: Debug,
{
    let rows = zset_to_rows(set);
    let mut data_rows = DataRows::with_capacity(&format, &order, rows.len());
    for row in rows {
        data_rows.push(row)
    }
    data_rows.get()
}

/// Version of hash that takes the result of orderby: a zset that is expected
/// to contain a single vector with all the data.
pub fn zset_of_vectors_to_strings<K, W>(
    set: &OrdZSet<Vec<K>, W>,
    format: String,
    order: SortOrder,
) -> Vec<Vec<String>>
where
    K: DBData + ToSqlRow,
    W: DBWeight + ZRingValue,
{
    let mut data_rows = DataRows::new(&format, &order);
    let mut cursor = set.cursor();
    while cursor.key_valid() {
        let w = cursor.weight();
        if w != W::one() {
            panic!("Weight is not one!");
        }
        let row_vec: Vec<K> = cursor.key().to_vec();
        let sql_rows = row_vec.iter().map(|k| k.to_row());
        for row in sql_rows {
            data_rows.push(row);
        }
        cursor.step_key();
    }
    data_rows.get()
}

/// Blow up a zset into multiple zsets, one for each "element"
pub fn to_elements<K, W>(set: &OrdZSet<K, W>) -> Vec<OrdZSet<K, W>>
where
    K: DBData,
    W: DBWeight,
{
    let mut cursor = set.cursor();
    let mut result = Vec::new();
    while cursor.key_valid() {
        let w = cursor.weight();
        let k = cursor.key();
        result.push(zset!(k.clone() => w));
        cursor.step_key();
    }
    result
}

/// This function mimics the md5 checksum computation from SqlLogicTest
/// The format is from the SqlLogicTest query output string format
pub fn hash<K, W>(set: &OrdZSet<K, W>, format: String, order: SortOrder) -> String
where
    K: DBData + ToSqlRow,
    W: DBWeight + ZRingValue,
    usize: TryFrom<W>,
    <usize as TryFrom<W>>::Error: Debug,
{
    let vec = zset_to_strings::<K, W>(set, format, order);
    let mut builder = String::default();
    for row in vec {
        for col in row {
            builder = builder + &col + "\n"
        }
    }
    // println!("{}", builder);
    let digest = md5::compute(builder);
    return format!("{:x}", digest);
}

/// Version of hash that takes the result of orderby: a zset that is expected
/// to contain a single vector with all the data.
pub fn hash_vectors<K, W>(set: &OrdZSet<Vec<K>, W>, format: String, order: SortOrder) -> String
where
    K: DBData + ToSqlRow,
    W: DBWeight + ZRingValue,
{
    // Result of orderby - there should be at most one row in the set.
    let mut builder = String::default();
    let mut cursor = set.cursor();
    while cursor.key_valid() {
        let w = cursor.weight();
        if w != W::one() {
            panic!("Weight is not one!");
        }
        let row_vec: Vec<K> = cursor.key().to_vec();
        let sql_rows = row_vec.iter().map(|k| k.to_row());
        let mut data_rows = DataRows::with_capacity(&format, &order, sql_rows.len());
        for row in sql_rows {
            data_rows.push(row);
        }
        for row in data_rows.get() {
            for col in row {
                builder = builder + &col + "\n"
            }
        }
        cursor.step_key();
    }
    // println!("{}", builder);
    let digest = md5::compute(builder);
    return format!("{:x}", digest);
}

// The count of elements in a zset that contains a vector is
// given by the count of the elements of the vector times the
// weight of the vector.
pub fn weighted_vector_count<K, W>(set: &OrdZSet<Vec<K>, W>) -> isize
where
    K: DBData + ToSqlRow,
    W: DBWeight + ZRingValue,
    isize: MulByRef<W, Output = isize>,
{
    let mut sum: isize = 0;
    let mut cursor = set.cursor();
    while cursor.key_valid() {
        let key = cursor.key();
        sum = sum + (key.len() as isize).mul_by_ref(&cursor.weight());
        cursor.step_key();
    }
    sum
}

// Check that two zsets are equal.  If yes, returns true.
// If not, print a diff of the zsets and returns false.
// Assumes that the zsets are positive (all weights are positive).
pub fn must_equal<K, W>(left: &OrdZSet<K, W>, right: &OrdZSet<K, W>) -> bool
where
    K: DBData + Clone,
    W: DBWeight + ZRingValue,
{
    let diff = left.add_by_ref(&right.neg_by_ref());
    if diff.is_zero() {
        return true;
    }
    let mut cursor = diff.cursor();
    while cursor.key_valid() {
        let key = cursor.key().clone();
        let weight = cursor.weight();
        if weight.le0() {
            println!("R: {:?}x{:?}", key, weight.neg());
        } else {
            println!("L: {:?}x{:?}", key, weight);
        }
        cursor.step_key();
    }
    false
}

fn jitset_to_map(data: &Vec<(RowLiteral, i32)>) -> BTreeMap<RowLiteral, i32> {
    let mut result = BTreeMap::new();
    for (r, w) in data.iter() {
        result.insert(r.clone(), *w);
    }
    result
}

fn subtract(
    left: &BTreeMap<RowLiteral, i32>,
    right: &BTreeMap<RowLiteral, i32>,
) -> BTreeMap<RowLiteral, i32> {
    let mut result = left.clone();
    for (r, v) in right.iter() {
        match result.get(r) {
            None => {
                result.insert(r.clone(), -*v);
            }
            Some(lv) => {
                let diff = lv - v;
                if diff == 0 {
                    result.remove(r);
                } else {
                    result.entry(r.clone()).and_modify(|lv| *lv = diff);
                }
            }
        }
    }
    result
}

// Check that two StreamCollections are equal.  If yes, returns true.
// If not, print a diff and returns false.
// Assumes that the literals are canonical and positive (all weights are
// positive).
pub fn must_equal_sc(left: &StreamCollection, right: &StreamCollection) -> bool {
    // println!("L: {:?}", left);
    // println!("R: {:?}", right);
    match (left, right) {
        (Set(left_rows), Set(right_rows)) => {
            let left = jitset_to_map(left_rows);
            let right = jitset_to_map(right_rows);
            let diff = subtract(&left, &right);
            for (r, v) in diff.iter() {
                if *v < 0 {
                    println!("R: {:?}x{:?}", r, -v);
                } else {
                    println!("L: {:?}x{:?}", r, v);
                }
            }
            diff.is_empty()
        }
        (Map(_left_rows), Map(_right_rows)) => {
            todo!()
        }
        _ => {
            println!("Collections of different types");
            false
        }
    }
}
