pub fn str_para_arr<const N: usize>(s: &str) -> [u8; N] {
    let mut arr = [0u8; N];
    let b = s.as_bytes();
    let n = b.len().min(N);
    arr[..n].copy_from_slice(&b[..n]);
    arr
}

pub fn arr_para_str(arr: &[u8]) -> String {
    let n = arr.iter().rposition(|&b| b != 0).map(|i| i + 1).unwrap_or(0);
    String::from_utf8_lossy(&arr[..n]).into_owned()
}
