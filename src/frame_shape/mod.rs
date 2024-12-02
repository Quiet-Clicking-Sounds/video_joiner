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
    /// see [readme_data/frame_shapes_9v.svg](../readme_data/frame_shapes_10.svg) for shape ref
    ExtendedLandscape,
    /// see [readme_data/frame_shapes_9v.svg](../readme_data/frame_shapes_11.svg) for shape ref
    ExtendedLandscape2,
}

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
            FrameShape::ExtendedLandscape2 => 8,
            FrameShape::ExtendedLandscape => 9,
        }
    }
}

