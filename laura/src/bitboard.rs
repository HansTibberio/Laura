use std::mem::transmute;
use std::fmt;

use crate::color::Color;
use crate::square::Square;


#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct BitBoard(pub u64);