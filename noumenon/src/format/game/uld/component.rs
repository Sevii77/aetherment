use std::io::{Read, Seek, Write};
use binrw::{binrw, BinRead, BinWrite};

#[binrw]
#[brw(little, repr = u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ComponentType {
	#[default] Custom = 0x0,
	Button = 0x1,
	Window = 0x2,
	CheckBox = 0x3,
	RadioButton = 0x4,
	Gauge = 0x5,
	Slider = 0x6,
	TextInput = 0x7,
	NumericInput = 0x8,
	List = 0x9, //?
	DropDown = 0xA,
	Tabbed = 0xB,
	TreeList = 0xC,
	ScrollBar = 0xD,
	ListItem = 0xE,
	Icon = 0xF,
	IconWithText = 0x10,
	DragDrop = 0x11,
	LeveCard = 0x12,
	NineGridText = 0x13,
	Journal = 0x14,
	Multipurpose = 0x15,
	Map = 0x16,
	Preview = 0x17,
	Unknown25 = 0x19,
}

// ---------------------------------------- //

#[derive(Debug, Clone, PartialEq)]
pub enum Component {
	Custom(Vec<u8>),
	Button(ButtonComponent),
	Window(WindowComponent),
	CheckBox(CheckBoxComponent),
	RadioButton(RadioButtonComponent),
	Gauge(GaugeComponent),
	Slider(SliderComponent),
	TextInput(TextInputComponent),
	NumericInput(NumericInputComponent),
	List(ListComponent),
	DropDown(DropDownComponent),
	Tabbed(TabComponent),
	TreeList(TreeListComponent),
	ScrollBar(ScrollBarComponent),
	ListItem(ListItemComponent),
	Icon(IconComponent),
	IconWithText(IconWithTextComponent),
	DragDrop(DragDropComponent),
	LeveCard(LeveCardComponent),
	NineGridText(NineGridComponent),
	Journal(JournalComponent),
	Multipurpose(MultipurposeComponent),
	Map(MapComponent),
	Preview(PreviewComponent),
	Unknown25(Unknown25Component),
}

impl Component {
	pub fn get_type(&self) -> ComponentType {
		match self {
			Component::Custom(_) => ComponentType::Custom,
			Component::Button(_) => ComponentType::Button,
			Component::Window(_) => ComponentType::Window,
			Component::CheckBox(_) => ComponentType::CheckBox,
			Component::RadioButton(_) => ComponentType::RadioButton,
			Component::Gauge(_) => ComponentType::Gauge,
			Component::Slider(_) => ComponentType::Slider,
			Component::TextInput(_) => ComponentType::TextInput,
			Component::NumericInput(_) => ComponentType::NumericInput,
			Component::List(_) => ComponentType::List,
			Component::DropDown(_) => ComponentType::DropDown,
			Component::Tabbed(_) => ComponentType::Tabbed,
			Component::TreeList(_) => ComponentType::TreeList,
			Component::ScrollBar(_) => ComponentType::ScrollBar,
			Component::ListItem(_) => ComponentType::ListItem,
			Component::Icon(_) => ComponentType::Icon,
			Component::IconWithText(_) => ComponentType::IconWithText,
			Component::DragDrop(_) => ComponentType::DragDrop,
			Component::LeveCard(_) => ComponentType::LeveCard,
			Component::NineGridText(_) => ComponentType::NineGridText,
			Component::Journal(_) => ComponentType::Journal,
			Component::Multipurpose(_) => ComponentType::Multipurpose,
			Component::Map(_) => ComponentType::Map,
			Component::Preview(_) => ComponentType::Preview,
			Component::Unknown25(_) => ComponentType::Unknown25,
		}
	}
}

impl Default for Component {
	fn default() -> Self {
		Component::Custom(Default::default())
	}
}

impl BinRead for Component {
	type Args<'a> = (ComponentType, u16);
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, (typ, size): Self::Args<'_>,) -> binrw::BinResult<Self> {
		Ok(match typ {
			ComponentType::Custom => Component::Custom(Vec::<u8>::read_options(reader, endian, binrw::VecArgs{count: size as usize, inner: ()})?),
			ComponentType::Button => Component::Button(ButtonComponent::read_options(reader, endian, ())?),
			ComponentType::Window => Component::Window(WindowComponent::read_options(reader, endian, ())?),
			ComponentType::CheckBox => Component::CheckBox(CheckBoxComponent::read_options(reader, endian, ())?),
			ComponentType::RadioButton => Component::RadioButton(RadioButtonComponent::read_options(reader, endian, ())?),
			ComponentType::Gauge => Component::Gauge(GaugeComponent::read_options(reader, endian, ())?),
			ComponentType::Slider => Component::Slider(SliderComponent::read_options(reader, endian, ())?),
			ComponentType::TextInput => Component::TextInput(TextInputComponent::read_options(reader, endian, ())?),
			ComponentType::NumericInput => Component::NumericInput(NumericInputComponent::read_options(reader, endian, ())?),
			ComponentType::List => Component::List(ListComponent::read_options(reader, endian, ())?),
			ComponentType::DropDown => Component::DropDown(DropDownComponent::read_options(reader, endian, ())?),
			ComponentType::Tabbed => Component::Tabbed(TabComponent::read_options(reader, endian, ())?),
			ComponentType::TreeList => Component::TreeList(TreeListComponent::read_options(reader, endian, ())?),
			ComponentType::ScrollBar => Component::ScrollBar(ScrollBarComponent::read_options(reader, endian, ())?),
			ComponentType::ListItem => Component::ListItem(ListItemComponent::read_options(reader, endian, ())?),
			ComponentType::Icon => Component::Icon(IconComponent::read_options(reader, endian, ())?),
			ComponentType::IconWithText => Component::IconWithText(IconWithTextComponent::read_options(reader, endian, ())?),
			ComponentType::DragDrop => Component::DragDrop(DragDropComponent::read_options(reader, endian, ())?),
			ComponentType::LeveCard => Component::LeveCard(LeveCardComponent::read_options(reader, endian, ())?),
			ComponentType::NineGridText => Component::NineGridText(NineGridComponent::read_options(reader, endian, ())?),
			ComponentType::Journal => Component::Journal(JournalComponent::read_options(reader, endian, ())?),
			ComponentType::Multipurpose => Component::Multipurpose(MultipurposeComponent::read_options(reader, endian, ())?),
			ComponentType::Map => Component::Map(MapComponent::read_options(reader, endian, ())?),
			ComponentType::Preview => Component::Preview(PreviewComponent::read_options(reader, endian, ())?),
			ComponentType::Unknown25 => Component::Unknown25(Unknown25Component::read_options(reader, endian, ())?),
		})
	}
}

impl BinWrite for Component {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		match self {
			Component::Custom(v) => v.write_options(writer, endian, ())?,
			Component::Button(v) => v.write_options(writer, endian, ())?,
			Component::Window(v) => v.write_options(writer, endian, ())?,
			Component::CheckBox(v) => v.write_options(writer, endian, ())?,
			Component::RadioButton(v) => v.write_options(writer, endian, ())?,
			Component::Gauge(v) => v.write_options(writer, endian, ())?,
			Component::Slider(v) => v.write_options(writer, endian, ())?,
			Component::TextInput(v) => v.write_options(writer, endian, ())?,
			Component::NumericInput(v) => v.write_options(writer, endian, ())?,
			Component::List(v) => v.write_options(writer, endian, ())?,
			Component::DropDown(v) => v.write_options(writer, endian, ())?,
			Component::Tabbed(v) => v.write_options(writer, endian, ())?,
			Component::TreeList(v) => v.write_options(writer, endian, ())?,
			Component::ScrollBar(v) => v.write_options(writer, endian, ())?,
			Component::ListItem(v) => v.write_options(writer, endian, ())?,
			Component::Icon(v) => v.write_options(writer, endian, ())?,
			Component::IconWithText(v) => v.write_options(writer, endian, ())?,
			Component::DragDrop(v) => v.write_options(writer, endian, ())?,
			Component::LeveCard(v) => v.write_options(writer, endian, ())?,
			Component::NineGridText(v) => v.write_options(writer, endian, ())?,
			Component::Journal(v) => v.write_options(writer, endian, ())?,
			Component::Multipurpose(v) => v.write_options(writer, endian, ())?,
			Component::Map(v) => v.write_options(writer, endian, ())?,
			Component::Preview(v) => v.write_options(writer, endian, ())?,
			Component::Unknown25(v) => v.write_options(writer, endian, ())?,
		}
		
		Ok(())
	}
}

// ---------------------------------------- //

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ButtonComponent {
	pub unk: [u32; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WindowComponent {
	pub unk: [u32; 8],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CheckBoxComponent {
	pub unk: [u32; 3],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RadioButtonComponent {
	pub unk: [u32; 4],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GaugeComponent {
	pub unk: [u32; 6],
	pub vertical_margin: u16,
	pub horizontal_margin: u16,
	#[br(map = |v: u8| v != 0)]
	#[bw(map = |v: &bool| if *v {1u8} else {0})]
	pub is_vertical: bool,
	pub padding: [u8; 3],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SliderComponent {
	pub unk: [u32; 4],
	#[br(map = |v: u8| v != 0)]
	#[bw(map = |v: &bool| if *v {1u8} else {0})]
	pub is_vertical: bool,
	pub left_offset: u8,
	pub right_offset: u8,
	pub padding: u8,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextInputComponent {
	pub unk: [u32; 16],
	pub color: u32,
	pub ime_color: u32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NumericInputComponent {
	pub unk: [u32; 5],
	pub color: u32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListComponent {
	pub unk: [u32; 5],
	pub wrap: u8,
	pub orientation: u8,
	pub padding: [u8; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DropDownComponent {
	pub unk: [u32; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TabComponent {
	pub unk: [u32; 4],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TreeListComponent {
	pub unk: [u32; 5],
	pub wrap: u8,
	pub orientation: u8,
	pub padding: [u8; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScrollBarComponent {
	pub unk: [u32; 4],
	pub margin: u16,
	#[br(map = |v: u8| v != 0)]
	#[bw(map = |v: &bool| if *v {1u8} else {0})]
	pub is_vertical: bool,
	pub padding: i8,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListItemComponent {
	pub unk: [u32; 4],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IconComponent {
	pub unk: [u32; 8],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IconWithTextComponent {
	pub unk: [u32; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DragDropComponent {
	pub unk: [u32; 1],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LeveCardComponent {
	pub unk: [u32; 3],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NineGridComponent {
	pub unk: [u32; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct JournalComponent {
	pub unk: [u32; 32],
	pub margin: u16,
	pub unk1: u16,
	pub unk2: u16,
	pub padding: u16,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MultipurposeComponent {
	pub unk: [u32; 3],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MapComponent {
	pub unk: [u32; 10],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PreviewComponent {
	pub unk: [u32; 2],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Unknown25Component {
	pub unk: [u32; 3],
}