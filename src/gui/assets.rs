//! Every icon and illustration the window draws, embedded in the binary.
//! Debug builds would otherwise read GPUI Kit's icons from the Cargo registry.
use gpui_kit::assets::{Assets, icon_assets};
use gpui_kit::{AssetSource, Result, SharedString};
use std::borrow::Cow;

icon_assets!(
    ExtraIcons,
    [
        Activity,
        Bot,
        Copy,
        SquareTerminal,
        ArrowLeft,
        ArrowRight,
        Check,
        ChevronDown,
        ChevronLeft,
        ChevronRight,
        CircleCheck,
        CircleDashed,
        CircleX,
        ClipboardCheck,
        Close,
        Cpu,
        Eye,
        EyeOff,
        FileText,
        Gpu,
        HardDrive,
        HardDriveDownload,
        Info,
        Layers,
        LockKeyhole,
        LogIn,
        MemoryStick,
        Minus,
        Monitor,
        MonitorPlay,
        Network,
        Palette,
        Play,
        Plus,
        Power,
        RefreshCw,
        RotateCcw,
        ShieldCheck,
        TriangleAlert,
        WindowClose,
        WindowMaximize,
        WindowMinimize,
        WindowRestore,
    ]
);

/// PNGs rendered from the SVG sources beside them with `resvg`; GPUI drops these SVGs.
const ILLUSTRATIONS: [(&str, &[u8]); 2] = [
    ("illustrations/aurora.png", include_bytes!("../../assets/illustrations/aurora.png")),
    ("illustrations/welcome.png", include_bytes!("../../assets/illustrations/welcome.png")),
];

pub(super) struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some((_, bytes)) = ILLUSTRATIONS.iter().find(|(name, _)| *name == path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        match ExtraIcons.load(path)? {
            Some(bytes) => Ok(Some(bytes)),
            None => Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let mut paths = Assets.list(path)?;
        paths.extend(ExtraIcons.list(path)?);
        paths.extend(ILLUSTRATIONS.iter().map(|(name, _)| *name).filter(|name| name.starts_with(path)).map(SharedString::from));
        paths.sort();
        paths.dedup();
        Ok(paths)
    }
}
