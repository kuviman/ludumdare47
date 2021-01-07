pub fn iter2<T>(a: [T; 2]) -> impl Iterator<Item = T> {
    let [a0, a1] = a;
    std::iter::once(a0).chain(std::iter::once(a1))
}

pub fn iter3<T>(a: [T; 3]) -> impl Iterator<Item = T> {
    let [a0, a1, a2] = a;
    std::iter::once(a0)
        .chain(std::iter::once(a1))
        .chain(std::iter::once(a2))
}
