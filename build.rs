/// This is a hack to support dynamic K values.
/// K values are implemented as a const generic in our code
/// as we expect it to remain constant across executions
/// and benefit from compile-time optimizations.
/// This build script will set the value of K at compile-time
/// from an environment variable, so one can easily build
/// the project "just in time" with the desired K value.
/// This will not re-build if the K value does not change.
fn build_constants() {
    let out_dir: std::path::PathBuf = std::env::var("OUT_DIR")
        .expect("Failed to obtain OUT_DIR")
        .into();
    let mut code = Vec::new();

    println!("cargo:rerun-if-env-changed=K");
    let k: usize = std::env::var("K")
        .unwrap_or_else(|_| "25".into())
        .parse()
        .expect("Failed to parse K");
    assert!(k >= 1, "K must be ≥ 1");
    assert!(k <= 59, "K must be ≤ 59");
    assert!(k % 2 == 1, "K must be odd");
    code.push(format!("pub const K: usize = {k};"));

    let kmer_bits = 2 * k;
    code.push(format!("pub const KMER_BITS: usize = {kmer_bits};"));

    let canon_bits = kmer_bits - 1;
    code.push(format!("pub const CANON_BITS: usize = {canon_bits};"));

    let kt = select_type(kmer_bits);
    code.push(format!("pub type KT = {kt};"));

    let pos_bits = canon_bits.next_power_of_two().ilog2() as usize;
    code.push(format!("pub const POS_BITS: usize = {pos_bits};"));

    let necklace_pos_bits = kmer_bits + pos_bits;
    code.push(format!("pub const N_BITS: usize = {necklace_pos_bits};"));

    let nt = select_type(necklace_pos_bits);
    code.push(format!("pub type NT = {nt};"));

    println!("cargo:rerun-if-env-changed=PREFIX_BITS");
    let prefix_bits: usize = std::env::var("PREFIX_BITS")
        .unwrap_or_else(|_| "24".into())
        .parse()
        .expect("Failed to parse PREFIX_BITS");
    assert!(prefix_bits >= 1, "PREFIX_BITS must be ≥ 1");
    assert!(
        prefix_bits < kmer_bits,
        "PREFIX_BITS must be < 2*K (here PREFIX_BITS={prefix_bits} ≥ 2*K={kmer_bits})"
    );
    code.push(format!("pub const PREFIX_BITS: usize = {prefix_bits};"));

    println!("cargo:rerun-if-env-changed=M");
    let m: usize = std::env::var("M")
        .unwrap_or_else(|_| "9".into())
        .parse()
        .expect("Failed to parse M");
    assert!(m >= 1, "M must be ≥ 1");
    assert!(m <= k, "M must be ≤ K (here M={m} > K={k})");
    code.push(format!("pub const M: usize = {m};"));

    std::fs::write(out_dir.join("constants.rs"), code.join("\n"))
        .expect("Failed to write const file");
}

fn select_type(n_bits: usize) -> &'static str {
    match n_bits.next_power_of_two() {
        1 | 2 | 4 | 8 => "u8",
        16 => "u16",
        32 => "u32",
        64 => "u64",
        128 => "u128",
        _ => panic!("Cannot fit {n_bits} bits in a primitive type"),
    }
}

fn build_ffi() -> miette::Result<()> {
    let path_src = std::path::PathBuf::from("src"); // include path
    let path_cxx = std::path::PathBuf::from("cxx"); // include path

    let mut build = autocxx_build::Builder::new("src/ffi.rs", [&path_src, &path_cxx])
        .extra_clang_args(&["-std=c++17"])
        .build()?;
    build
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-#pragma-messages")
        .compile("autocxx-demo"); // arbitrary library name, pick anything
    println!("cargo:rerun-if-changed=src/ffi.rs");

    Ok(())
}

fn main() -> miette::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    build_constants();
    build_ffi()
}
