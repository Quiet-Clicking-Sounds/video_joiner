use crate::request_input;

mod audio;
mod frame_join;

#[derive(Clone, Debug)]
pub(crate) enum FrameShape {
    Mono,
    /// see [readme_data/frame_shapes_1.svg](../readme_data/frame_shapes_1.svg) for shape ref
    Dual,
    /// see [readme_data/frame_shapes_2.svg](../readme_data/frame_shapes_2.svg) for shape ref
    Triple,
    /// see [readme_data/frame_shapes_3.svg](../readme_data/frame_shapes_3.svg) for shape ref
    Quad,
    /// see [readme_data/frame_shapes_4.svg](../readme_data/frame_shapes_4.svg) for shape ref
    VertEmph,
    /// see [readme_data/frame_shapes_5.svg](../readme_data/frame_shapes_5.svg) for shape ref
    HorizEmph,
    /// see [readme_data/frame_shapes_6.svg](../readme_data/frame_shapes_6.svg) for shape ref
    VertEmph2,
    /// see [readme_data/frame_shapes_6.svg](../readme_data/frame_shapes_6.svg) for shape ref
    HorizEmph2,
    /// see [readme_data/frame_shapes_7.svg](../readme_data/frame_shapes_7.svg) for shape ref
    SideVert,
    /// see [readme_data/frame_shapes_7.svg](../readme_data/frame_shapes_7.svg) for shape ref
    SideVert2,
    /// see [readme_data/frame_shapes_8.svg](../readme_data/frame_shapes_8.svg) for shape ref
    CentreEmphVert,
    /// see [readme_data/frame_shapes_8v.svg](../readme_data/frame_shapes_8v.svg) for shape ref
    CentreEmphVert2,
    /// see [readme_data/frame_shapes_9.svg](../readme_data/frame_shapes_8v.svg) for shape ref
    MoreHoriz,
    /// see [readme_data/frame_shapes_9v.svg](../readme_data/frame_shapes_8v.svg) for shape ref
    MoreHoriz2,
    /// see [readme_data/frame_shapes_10.svg](../readme_data/frame_shapes_10.svg) for shape ref
    ExtendedLandscape,
    /// see [readme_data/frame_shapes_11.svg](../readme_data/frame_shapes_11.svg) for shape ref
    ExtendedLandscape2,
    /// see [readme_data/frame_shapes_12.svg](../readme_data/frame_shapes_12.svg) for shape ref
    OffsetVH4x4,
}

static FRAME_SHAPE_MAPPING: [(FrameShape, [&str; 3], &str); 17] = [
    (FrameShape::Mono, ["1", "Mono", "M"], ""),
    (FrameShape::Dual, ["2", "Double", "D"], "../readme_data/frame_shapes_1.svg"),
    (FrameShape::Triple, ["3", "Triple", "T"], "../readme_data/frame_shapes_2.svg"),
    (FrameShape::Quad, ["4", "Quad", "Q"], "../readme_data/frame_shapes_3.svg"),
    (FrameShape::VertEmph, ["5", "VertEmph", "V"], "../readme_data/frame_shapes_4.svg"),
    (FrameShape::HorizEmph, ["6", "HorizEmph", "H"], "../readme_data/frame_shapes_5.svg"),
    (FrameShape::VertEmph2, ["7", "VertEmph2", "V2"], "../readme_data/frame_shapes_6.svg"),
    (FrameShape::HorizEmph2, ["8", "HorizEmph2", "H2"], "../readme_data/frame_shapes_6.svg"),
    (FrameShape::SideVert, ["9", "SideVert", "S"], "../readme_data/frame_shapes_7.svg"),
    (FrameShape::SideVert2, ["10", "SideVert2", "S2"], "../readme_data/frame_shapes_7.svg"),
    (FrameShape::CentreEmphVert, ["11", "CentreEmphVert", "CE"], "../readme_data/frame_shapes_8.svg"),
    (FrameShape::CentreEmphVert2, ["12", "CentreEmphVert2", "CE2"], "../readme_data/frame_shapes_8v.svg"),
    (FrameShape::MoreHoriz, ["13", "MoreHoriz", "MH"], "../readme_data/frame_shapes_8v.svg"),
    (FrameShape::MoreHoriz2, ["14", "MoreHoriz2", "MH2"], "../readme_data/frame_shapes_8v.svg"),
    (FrameShape::ExtendedLandscape, ["15", "ExtendedLandscape", "EL"], "../readme_data/frame_shapes_10.svg"),
    (FrameShape::ExtendedLandscape2, ["16", "ExtendedLandscape2", "EL2"], "../readme_data/frame_shapes_11.svg"),
    (FrameShape::OffsetVH4x4, ["17", "OffsetVH", "4x4"], "../readme_data/frame_shapes_12.svg"),
];

impl FrameShape {
    pub(crate) fn count(&self) -> u32 {
        match self {
            FrameShape::Mono => 1,
            FrameShape::Dual => 2,
            FrameShape::Triple => 3,
            FrameShape::Quad => 4,
            FrameShape::VertEmph | FrameShape::VertEmph2 |
            FrameShape::CentreEmphVert | FrameShape::CentreEmphVert2 => 5,
            FrameShape::HorizEmph | FrameShape::HorizEmph2 => 4,
            FrameShape::SideVert | FrameShape::SideVert2 => 3,
            FrameShape::MoreHoriz | FrameShape::MoreHoriz2 => 7,
            FrameShape::ExtendedLandscape2 | FrameShape::OffsetVH4x4 => 8,
            FrameShape::ExtendedLandscape => 9,
        }
    }
    pub(crate) fn from_str_opt(data: Option<String>) -> Self {
        let data = match data {
            None => { request_input("Split Format 'Double' / 'Triple' / 'Quad' (see README.md for more options): ") }
            Some(data) => { data }
        }.to_lowercase();
        let data = data.as_str().trim();
        for (fs, x, _) in FRAME_SHAPE_MAPPING.iter() {
            if x.iter().any(|d| data.eq_ignore_ascii_case(d)) {
                return fs.clone();
            }
        }

        Self::print_frame_mapping();
        panic!("No match found for split format: {}", data);
    }
    pub(crate) fn print_frame_mapping() {
        println!("FrameShape name mapping, use any of the below names after -s or -shape; \
        the program will ignore case");
        println!(
            "{:24}  {:2} {:24} {:2} \t {}",
            "FrameShapeName",
            "id", "LongName", "ShortName",
            "frame diagram");
        for (fs, names, info) in FRAME_SHAPE_MAPPING.iter() {
            println!(
                "{:24}  {:2} {:24} {:2} \t {}",
                format!("{:?}", fs),
                names[0], names[1], names[2],
                info)
        }
    }
}
