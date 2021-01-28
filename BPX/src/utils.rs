pub fn hash(s: &str) -> u64
{
    let mut val: u64 = 5381;

    for v in s.as_bytes()
    {
        val = ((val << 5) + val) + *v as u64;
    }
    return val;
}
