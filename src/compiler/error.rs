/// 编译期间产生的错误
#[derive(Debug)]
pub enum Error {
    /// no more bytes
    EOF,
    /// illegal Token
    IllegalToken
}
