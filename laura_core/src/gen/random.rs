
/// `Xoshiro256PlusPlus` is an implementation of the Xoshiro256++ pseudorandom number
/// generator (PRNG). It uses a 256-bit state (represented by four `u64` values)
/// and produces high-quality 64-bit pseudorandom numbers. This PRNG is known for
/// its high speed and good statistical properties.
///
/// The implementation relies on bitwise operations and shifts to evolve the internal
/// state and generate pseudorandom outputs.
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
    pub fn next(&mut self) -> u64 {
        let result: u64 = Self::rotl(self.state[0].wrapping_add(self.state[3]), 23).wrapping_add(self.state[0]);

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

#[test]
fn test_prng() {
    let seed: [u64; 4] = [0x1a2b3c4d5e6f7, 0x1122334455667788, 0x99aabbccddeeff00, 0x2233445566778899];
    let mut rng: Xoshiro256PlusPlus = Xoshiro256PlusPlus::new(seed);

    for _ in 0..10 {
        println!("{}", rng.next());
    }
}