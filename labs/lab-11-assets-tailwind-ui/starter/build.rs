fn main() {
    // TODO(lab-11): Stage the Lucide Iconify set before compiling icon declarations.
    topcoat::icon::iconify::BuildConfig::new()
        .icon_set("lucide")
        .stage()
        .unwrap();
    // TODO(lab-11): Generate Tailwind from the stylesheet installed by `topcoat ui init`.
    topcoat::tailwind::BuildConfig::new()
        .input("styles.css")
        .render()
        .unwrap();
}
