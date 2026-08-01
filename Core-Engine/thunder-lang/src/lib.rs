// ---------------------------------------------------------------------------
//  Thunder Blockchain — ThunderScript Language Library
// ---------------------------------------------------------------------------

pub mod ast;
pub mod compiler;
pub mod lexer;
pub mod parser;

/// Compile ThunderScript source code into VM bytecode.
pub use compiler::compile_source;
