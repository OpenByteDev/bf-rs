//! Just-in-time compiles Brainfuck AST to x64 machine code (`--features jit`, nightly only)
//!
//! This uses the [`dynasm`](https://crates.io/search?q=dynasm) crate to generate x86-64
//! machine code from peephole-optimized AST. This is currently the fastest implementation,
//! but it is available only on nightly Rust because `dynasm` uses a plugin.
//!
//! In the `bfi` interpreter, this pass is enabled by the `--jit` flag.
//! To go even faster, pass the `--unchecked` flag to the `bfi` interpreter to disable
//! memory bounds checking in the generated code. Note that this runs Brainfuck in
//! unsafe mode, which means that programs that move the pointer outside the allocated
//! memory will access and possibly overwrite arbitrary memory locations.

mod compiler;
mod rts;

pub use self::compiler::*;

use dynasmrt;

/// The representation of a JIT-compiled program.
///
/// Relies on the `dynasmrt` run-time system. [This
/// representation](../../src/bf/jit/mod.rs.html#26-29)
/// is from [the `dynlib-rs` tutorial].
///
/// [the `dynlib-rs` tutorial]: https://censoredusername.github.io/dynasm-rs/language/tutorial.html#advanced-usage
pub struct Program {
    code: dynasmrt::ExecutableBuffer,
    start: dynasmrt::AssemblyOffset,
}

#[cfg(test)]
mod tests {
    use test_helpers::*;
    use common::{BfResult, Error};

    #[test]
    fn move_right_once() {
        assert_parse_interpret(b">", "", Ok(""));
    }

    #[test]
    fn move_left_once() {
        assert_parse_interpret(b"<", "", Err(Error::PointerUnderflow));
    }

    #[test]
    fn move_right_forever() {
        assert_parse_interpret(b"+[>+]", "", Err(Error::PointerOverflow));
    }

    #[test]
    fn echo_one_byte() {
        assert_parse_interpret(b",.", "A", Ok("A"));
    }

    #[test]
    fn inc_echo_one_byte() {
        assert_parse_interpret(b",+.", "A", Ok("B"));
    }

    #[test]
    fn hello_world() {
        assert_parse_interpret(HELLO_WORLD_SRC, "", Ok("Hello, World!"));
    }

    #[test]
    fn factoring() {
        assert_parse_interpret(FACTOR_SRC, "2\n", Ok("2: 2\n"));
        assert_parse_interpret(FACTOR_SRC, "3\n", Ok("3: 3\n"));
        assert_parse_interpret(FACTOR_SRC, "6\n", Ok("6: 2 3\n"));
        assert_parse_interpret(FACTOR_SRC, "100\n", Ok("100: 2 2 5 5\n"));
    }

    fn assert_parse_interpret(program: &[u8], input: &str, output: BfResult<&str>) {
        let program = ::ast::parse_program(program).unwrap();
        let program = ::rle::compile(&program);
        let program = ::peephole::compile(&program);
        let program = ::jit::compile(&program, true);
        assert_interpret_result(&program, input.as_bytes(), output.map(|s| s.as_bytes()));
    }
}

