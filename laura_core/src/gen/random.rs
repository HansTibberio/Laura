/// `Xoshiro256PlusPlus` is an implementation of the Xoshiro256++ pseudorandom number
/// generator (PRNG). It uses a 256-bit state (represented by four `u64` values)
/// and produces high-quality 64-bit pseudorandom numbers. This PRNG is known for
/// its high speed and good statistical properties.
///
/// The implementation relies on bitwise operations and shifts to evolve the internal
/// state and generate pseudorandom outputs.
#[derive(Clone, Copy, Debug)]
pub struct Xoshiro256PlusPlus {
    state: [u64; 4],
}

impl Xoshiro256PlusPlus {
    /// Initializes a new instance of `Xoshiro256PlusPlus` with a given seed.
    pub fn new(seed: [u64; 4]) -> Self {
        Self { state: seed }
    }

    /// Rotates the bits of a 64-bit unsigned integer `x` to the left by `k` positions.
    /// The rotation is performed using a bitwise shift and OR operation.
    #[inline(always)]
    pub const fn rotl(x: u64, k: u64) -> u64 {
        (x << k) | (x >> (64 - k))
    }

    /// Generates the next 64-bit pseudorandom number using the current internal state
    /// and updates the state for subsequent calls.
    ///
    /// The algorithm applies a series of bitwise XOR, shifts, and rotations to produce
    /// a high-quality random number and ensure that the internal state evolves properly.
    pub fn next_u64(&mut self) -> u64 {
        let result: u64 =
            Self::rotl(self.state[0].wrapping_add(self.state[3]), 23).wrapping_add(self.state[0]);

        let t: u64 = self.state[1] << 17;

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = Self::rotl(self.state[3], 45);

        result
    }
}

impl Default for Xoshiro256PlusPlus {
    /// Initializes a new instance of `Xoshiro256PlusPlus` with a default seed.
    /// The seed is the first 64 bits of the decimal part of the square roots of the first prime numbers in the interval `[2, 7]`.
    fn default() -> Self {
        Self {
            state: [
                0x6A09_E667_F3BC_C908,
                0xBB67_AE85_84CA_A73B,
                0x3C6E_F372_FE94_F82B,
                0xA54F_F53A_5F1D_36F1,
            ],
        }
    }
}

#[test]
fn test_prng_seed() {
    let seed: [u64; 4] = [
        0x0001_A2B3_C4D5_E6F7,
        0x1122_3344_5566_7788,
        0x99AA_BBCC_DDEE_FF00,
        0x2233_4455_6677_8899,
    ];
    let mut prng: Xoshiro256PlusPlus = Xoshiro256PlusPlus::new(seed);

    for _ in 0..10 {
        println!("{}", prng.next_u64());
    }
}

#[test]
fn test_prng_default() {
    let prng_test: [u64; 4] = [
        0x3B33_5367_F044_75F5,
        0x42BB_AF82_469E_8642,
        0x258D_4A00_A40A_97E4,
        0xA44A_415D_5AA2_F14D,
    ];

    let mut prng: Xoshiro256PlusPlus = Xoshiro256PlusPlus::default();

    for random in prng_test {
        let prng: u64 = prng.next_u64();
        println!("{}", prng);
        assert_eq!(random, prng);
    }
}
