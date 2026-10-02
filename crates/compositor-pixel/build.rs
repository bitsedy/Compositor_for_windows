fn main() {
    println!("cargo:rerun-if-changed=c_src");

    let mut build = cc::Build::new();
    build
        .include("c_src")
        .file("c_src/AdjustPixels.c")
        .file("c_src/BrushPixels.c")
        .file("c_src/ContentFill.c")
        .file("c_src/DitherPixels.c")
        .file("c_src/HealPixels.c")
        .file("c_src/LensPixels.c")
        .file("c_src/LevelsPixels.c")
        .file("c_src/NoisePixels.c")
        .file("c_src/WandPixels.c")
        .opt_level(3);

    // On MSVC, suppress some legacy C warnings and ensure standard math behavior
    if build.get_compiler().is_like_msvc() {
        build.flag("/fp:precise");
        build.flag("/wd4244"); // conversion loss of data
        build.flag("/wd4267"); // conversion from size_t
    }

    build.compile("compositor_c_kernels");
}
