pub trait SocketOption {
    type Value;
    const level: c_int;
    const option: c_int;

    fn init() -> Self::Value;

    fn size(value: &Self::Value) -> usize
}
