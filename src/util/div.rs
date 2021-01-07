pub fn div_down(a: i64, b: i64) -> i64 {
    if a < 0 {
        return -div_up(-a, b);
    }
    if b < 0 {
        return -div_up(a, -b);
    }
    return a / b;
}
pub fn div_up(a: i64, b: i64) -> i64 {
    if a < 0 {
        return -div_down(-a, b);
    }
    if b < 0 {
        return -div_down(a, -b);
    }
    return (a + b - 1) / b;
}
