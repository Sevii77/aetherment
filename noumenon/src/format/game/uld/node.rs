use std::io::{Read, Seek, Write, SeekFrom};
use binrw::{binrw, BinRead, BinWrite};

#[binrw]
#[brw(little, repr = u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FontType {
	#[default] Axis = 0x0,
	MiedingerMed = 0x1,
	Miedinger = 0x2,
	TrumpGothic = 0x3,
	Jupiter = 0x4,
	JupiterLarge = 0x5,
	// TODO: check if custom fonts can be added some way
}

#[binrw]
#[brw(little, repr = u16)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CollisionType {
	#[default] Hit = 0x0,
	Focus = 0x1,
	Move = 0x2,
}

#[binrw]
#[brw(little, repr = u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GridPartsType {
	#[default] Divide = 0x0,
	Compose = 0x1,
}

#[binrw]
#[brw(little, repr = u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GridRenderType {
	#[default] Scale = 0x0,
	Tile = 0x1,
}

#[binrw]
#[brw(little, repr = u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SheetType {
	#[default] Addon = 0x0,
	Lobby = 0x1,
}

// ---------------------------------------- //

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeData {
	pub node_id: u32,
	pub parent_id: i32,
	pub next_sibling_id: i32,
	pub prev_sibling_id: i32,
	pub child_node_id: i32,
	// node_type: u32, // i32
	// _node_size: u16,
	pub tab_index: i16,
	pub unk1: [i32; 4],
	pub x: i16,
	pub y: i16,
	pub w: u16,
	pub h: u16,
	pub rotation: f32,
	pub scale_x: f32,
	pub scale_y: f32,
	pub origin_x: i16,
	pub origin_y: i16,
	pub priority: u16,
	
	pub visible: bool,
	pub enabled: bool,
	pub clip: bool,
	pub fill: bool,
	pub anchor_top: bool,
	pub anchor_bottom: bool,
	pub anchor_left: bool,
	pub anchor_right: bool,
	
	pub unk2: u8,
	pub multiply_red: i16,
	pub multiply_green: i16,
	pub multiply_blue: i16,
	pub add_red: i16,
	pub add_green: i16,
	pub add_blue: i16,
	pub alpha: u8,
	pub clip_count: u8,
	pub timeline_id: u16,
	pub node: Node,
}

impl BinRead for NodeData {
	type Args<'a> = &'a [super::UldComponent];
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, components: Self::Args<'_>,) -> binrw::BinResult<Self> {
		let pos = reader.stream_position()?;
		
		let node_id = u32::read_options(reader, endian, ())?;
		let parent_id = i32::read_options(reader, endian, ())?;
		let next_sibling_id = i32::read_options(reader, endian, ())?;
		let prev_sibling_id = i32::read_options(reader, endian, ())?;
		let child_node_id = i32::read_options(reader, endian, ())?;
		let node_type = u32::read_options(reader, endian, ())?;
		let node_size = u16::read_options(reader, endian, ())?;
		let tab_index = i16::read_options(reader, endian, ())?;
		let unk1 = [i32::read_options(reader, endian, ())?, i32::read_options(reader, endian, ())?, i32::read_options(reader, endian, ())?, i32::read_options(reader, endian, ())?];
		let x = i16::read_options(reader, endian, ())?;
		let y = i16::read_options(reader, endian, ())?;
		let w = u16::read_options(reader, endian, ())?;
		let h = u16::read_options(reader, endian, ())?;
		let rotation = f32::read_options(reader, endian, ())?;
		let scale_x = f32::read_options(reader, endian, ())?;
		let scale_y = f32::read_options(reader, endian, ())?;
		let origin_x = i16::read_options(reader, endian, ())?;
		let origin_y = i16::read_options(reader, endian, ())?;
		let priority = u16::read_options(reader, endian, ())?;
		// let field1 = NodeDataField1::read_options(reader, endian, ())?;
		let field1 = u8::read_options(reader, endian, ())?;
		let unk2 = u8::read_options(reader, endian, ())?;
		let multiply_red = i16::read_options(reader, endian, ())?;
		let multiply_green = i16::read_options(reader, endian, ())?;
		let multiply_blue = i16::read_options(reader, endian, ())?;
		let add_red = i16::read_options(reader, endian, ())?;
		let add_green = i16::read_options(reader, endian, ())?;
		let add_blue = i16::read_options(reader, endian, ())?;
		let alpha = u8::read_options(reader, endian, ())?;
		let clip_count = u8::read_options(reader, endian, ())?;
		let timeline_id = u16::read_options(reader, endian, ())?;
		let node = Node::read_options(reader, endian, (node_type, node_size, components))?;
		
		reader.seek(SeekFrom::Start(pos + node_size as u64))?;
		
		let visible = (field1 & 0x80) == 0x80;
		let enabled = (field1 & 0x40) == 0x40;
		let clip = (field1 & 0x20) == 0x20;
		let fill = (field1 & 0x10) == 0x10;
		let anchor_top = (field1 & 0x08) == 0x08;
		let anchor_bottom = (field1 & 0x04) == 0x04;
		let anchor_left = (field1 & 0x02) == 0x02;
		let anchor_right = (field1 & 0x01) == 0x01;
		
		Ok(Self {
			node_id,
			parent_id,
			next_sibling_id,
			prev_sibling_id,
			child_node_id,
			// node_type,
			// _node_size: node_size,
			tab_index,
			unk1,
			x,
			y,
			w,
			h,
			rotation,
			scale_x,
			scale_y,
			origin_x,
			origin_y,
			priority,
			// field1,
			visible,
			enabled,
			clip,
			fill,
			anchor_top,
			anchor_bottom,
			anchor_left,
			anchor_right,
			
			unk2,
			multiply_red,
			multiply_green,
			multiply_blue,
			add_red,
			add_green,
			add_blue,
			alpha,
			clip_count,
			timeline_id,
			node
		})
	}
}

impl BinWrite for NodeData {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		self.node_id.write_options(writer, endian, ())?;
		self.parent_id.write_options(writer, endian, ())?;
		self.next_sibling_id.write_options(writer, endian, ())?;
		self.prev_sibling_id.write_options(writer, endian, ())?;
		self.child_node_id.write_options(writer, endian, ())?;
		// self.node_type.write_options(writer, endian, ())?;
		self.node.get_type().write_options(writer, endian, ())?;
		let pos = writer.stream_position()?;
		0u16.write_options(writer, endian, ())?;
		self.tab_index.write_options(writer, endian, ())?;
		self.unk1[0].write_options(writer, endian, ())?;
		self.unk1[1].write_options(writer, endian, ())?;
		self.unk1[2].write_options(writer, endian, ())?;
		self.unk1[3].write_options(writer, endian, ())?;
		self.x.write_options(writer, endian, ())?;
		self.y.write_options(writer, endian, ())?;
		self.w.write_options(writer, endian, ())?;
		self.h.write_options(writer, endian, ())?;
		self.rotation.write_options(writer, endian, ())?;
		self.scale_x.write_options(writer, endian, ())?;
		self.scale_y.write_options(writer, endian, ())?;
		self.origin_x.write_options(writer, endian, ())?;
		self.origin_y.write_options(writer, endian, ())?;
		self.priority.write_options(writer, endian, ())?;
		// self.field1.write_options(writer, endian, ())?;
		let field1 = (self.visible as u8) << 7 | (self.enabled as u8) << 6 | (self.clip as u8) << 5 | (self.fill as u8) << 4 | (self.anchor_top as u8) << 3 | (self.anchor_bottom as u8) << 2 | (self.anchor_left as u8) << 1 | (self.anchor_right as u8);
		field1.write_options(writer, endian, ())?;
		self.unk2.write_options(writer, endian, ())?;
		self.multiply_red.write_options(writer, endian, ())?;
		self.multiply_green.write_options(writer, endian, ())?;
		self.multiply_blue.write_options(writer, endian, ())?;
		self.add_red.write_options(writer, endian, ())?;
		self.add_green.write_options(writer, endian, ())?;
		self.add_blue.write_options(writer, endian, ())?;
		self.alpha.write_options(writer, endian, ())?;
		self.clip_count.write_options(writer, endian, ())?;
		self.timeline_id.write_options(writer, endian, ())?;
		self.node.write_options(writer, endian, ())?;
		
		let end = writer.stream_position()?;
		writer.seek(SeekFrom::Start(pos))?;
		((end - pos) as u16).write_options(writer, endian, ())?;
		writer.seek(SeekFrom::Start(end))?;
		
		Ok(())
	}
}

// ---------------------------------------- //

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
	Image(ImageNode),
	Text(TextNode),
	NineGrid(NineGridNode),
	Counter(CounterNode),
	Collision(CollisionNode),
	Component(ComponentNode),
	Unknown(Vec<u8>),
	Other(u32),
}

impl Node {
	pub fn get_type(&self) -> u32 {
		match self {
			Node::Image(_) => 2,
			Node::Text(_) => 3,
			Node::NineGrid(_) => 4,
			Node::Counter(_) => 5,
			Node::Collision(_) => 8,
			Node::Component(v) => v.component_id,
			Node::Unknown(_) => 0,
			Node::Other(v) => *v,
		}
	}
}

impl Default for Node {
	fn default() -> Self {
		Node::Image(Default::default())
	}
}

impl BinRead for Node {
	type Args<'a> = (u32, u16, &'a [super::UldComponent]);
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, (node_type, node_size, components): Self::Args<'_>,) -> binrw::BinResult<Self> {
		Ok(match node_type {
			1 => Node::Other(node_type),
			2 => Node::Image(ImageNode::read_options(reader, endian, ())?),
			3 => Node::Text(TextNode::read_options(reader, endian, ())?),
			4 => Node::NineGrid(NineGridNode::read_options(reader, endian, ())?),
			5 => Node::Counter(CounterNode::read_options(reader, endian, ())?),
			8 => Node::Collision(CollisionNode::read_options(reader, endian, ())?),
			_ => {
				if node_size <= 88 {
					Node::Other(node_type)
				} else if node_type > 1000 {
					let component_type = components.iter().find(|c| c.id == node_type).map_or(super::ComponentType::Custom, |v| v.component.get_type());
					Node::Component(ComponentNode::read_options(reader, endian, (component_type, node_type))?)
				} else {
					let mut unknown_data = vec![0u8; node_size as usize - 88];
					reader.read_exact(&mut unknown_data)?;
					Node::Unknown(unknown_data)
				}
			}
		})
	}
}

impl BinWrite for Node {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		match self {
			Node::Image(v) => v.write_options(writer, endian, ())?,
			Node::Text(v) => v.write_options(writer, endian, ())?,
			Node::NineGrid(v) => v.write_options(writer, endian, ())?,
			Node::Counter(v) => v.write_options(writer, endian, ())?,
			Node::Collision(v) => v.write_options(writer, endian, ())?,
			Node::Component(v) => v.write_options(writer, endian, ())?,
			Node::Unknown(v) => writer.write_all(v)?,
			_ => (),
		}
		
		Ok(())
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentNodeNode {
	Button(ButtonComponentNode),
	Window(WindowComponentNode),
	CheckBox(CheckBoxComponentNode),
	RadioButton(RadioButtonComponentNode),
	Gauge(GaugeComponentNode),
	Slider(SliderComponentNode),
	TextInput(TextInputComponentNode),
	NumericInput(NumericInputComponentNode),
	List(ListComponentNode),
	// DropDown(DropDownComponentNode),
	Tabbed(TabbedComponentNode),
	// TreeList(TreeListComponentNode),
	// ScrollBar(ScrollBarComponentNode),
	ListItem(ListItemComponentNode),
	// Icon(IconComponentNode),
	// IconWithText(IconWithTextComponentNode),
	// DragDrop(DragDropComponentNode),
	// LeveCard(LeveCardComponentNode),
	NineGridText(NineGridTextComponentNode),
	// Journal(JournalComponentNode),
	// Multipurpose(MultipurposeComponentNode),
	// Map(MapComponentNode),
	// Preview(PreviewComponentNode),
	None,
}

impl ComponentNodeNode {
	pub fn get_type(&self) -> super::ComponentType {
		match self {
			ComponentNodeNode::Button(_) => super::ComponentType::Button,
			ComponentNodeNode::Window(_) => super::ComponentType::Window,
			ComponentNodeNode::CheckBox(_) => super::ComponentType::CheckBox,
			ComponentNodeNode::RadioButton(_) => super::ComponentType::RadioButton,
			ComponentNodeNode::Gauge(_) => super::ComponentType::Gauge,
			ComponentNodeNode::Slider(_) => super::ComponentType::Slider,
			ComponentNodeNode::TextInput(_) => super::ComponentType::TextInput,
			ComponentNodeNode::NumericInput(_) => super::ComponentType::NumericInput,
			ComponentNodeNode::List(_) => super::ComponentType::List,
			ComponentNodeNode::Tabbed(_) => super::ComponentType::Tabbed,
			ComponentNodeNode::ListItem(_) => super::ComponentType::ListItem,
			ComponentNodeNode::NineGridText(_) => super::ComponentType::NineGridText,
			_ => super::ComponentType::Custom,
		}
	}
}

impl Default for ComponentNodeNode {
	fn default() -> Self {
		ComponentNodeNode::Button(Default::default())
	}
}

impl BinRead for ComponentNodeNode {
	type Args<'a> = (super::ComponentType,);
	
	fn read_options<R: Read + Seek>(reader: &mut R, endian: binrw::Endian, (typ,): Self::Args<'_>,) -> binrw::BinResult<Self> {
		Ok(match typ {
			super::ComponentType::Button => ComponentNodeNode::Button(ButtonComponentNode::read_options(reader, endian, ())?),
			super::ComponentType::Window => ComponentNodeNode::Window(WindowComponentNode::read_options(reader, endian, ())?),
			super::ComponentType::CheckBox => ComponentNodeNode::CheckBox(CheckBoxComponentNode::read_options(reader, endian, ())?),
			super::ComponentType::RadioButton => ComponentNodeNode::RadioButton(RadioButtonComponentNode::read_options(reader, endian, ())?),
			super::ComponentType::Gauge => ComponentNodeNode::Gauge(GaugeComponentNode::read_options(reader, endian, ())?),
			super::ComponentType::Slider => ComponentNodeNode::Slider(SliderComponentNode::read_options(reader, endian, ())?),
			super::ComponentType::TextInput => ComponentNodeNode::TextInput(TextInputComponentNode::read_options(reader, endian, ())?),
			super::ComponentType::NumericInput => ComponentNodeNode::NumericInput(NumericInputComponentNode::read_options(reader, endian, ())?),
			super::ComponentType::List => ComponentNodeNode::List(ListComponentNode::read_options(reader, endian, ())?),
			super::ComponentType::Tabbed => ComponentNodeNode::Tabbed(TabbedComponentNode::read_options(reader, endian, ())?),
			super::ComponentType::ListItem => ComponentNodeNode::ListItem(ListItemComponentNode::read_options(reader, endian, ())?),
			super::ComponentType::NineGridText => ComponentNodeNode::NineGridText(NineGridTextComponentNode::read_options(reader, endian, ())?),
			_ => ComponentNodeNode::None,
		})
	}
}

impl BinWrite for ComponentNodeNode {
	type Args<'a> = ();
	
	fn write_options<W: Write + Seek>(&self, writer: &mut W, endian: binrw::Endian, _args: Self::Args<'_>,) -> binrw::BinResult<()> {
		match self {
			ComponentNodeNode::Button(v) => v.write_options(writer, endian, ())?,
			ComponentNodeNode::Window(v) => v.write_options(writer, endian, ())?,
			ComponentNodeNode::CheckBox(v) => v.write_options(writer, endian, ())?,
			ComponentNodeNode::RadioButton(v) => v.write_options(writer, endian, ())?,
			ComponentNodeNode::Gauge(v) => v.write_options(writer, endian, ())?,
			ComponentNodeNode::Slider(v) => v.write_options(writer, endian, ())?,
			ComponentNodeNode::TextInput(v) => v.write_options(writer, endian, ())?,
			ComponentNodeNode::NumericInput(v) => v.write_options(writer, endian, ())?,
			ComponentNodeNode::List(v) => v.write_options(writer, endian, ())?,
			ComponentNodeNode::Tabbed(v) => v.write_options(writer, endian, ())?,
			ComponentNodeNode::ListItem(v) => v.write_options(writer, endian, ())?,
			ComponentNodeNode::NineGridText(v) => v.write_options(writer, endian, ())?,
			_ => (),
		}
		
		Ok(())
	}
}

// ---------------------------------------- //

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImageNode {
	pub part_list_id: u32,
	pub part_id: u32,
	#[br(map = |v: u8| v != 0)]
	#[bw(map = |v: &bool| if *v {1u8} else {0})]
	pub flip_h: bool,
	#[br(map = |v: u8| v != 0)]
	#[bw(map = |v: &bool| if *v {1u8} else {0})]
	pub flip_v: bool,
	pub wrap: u8,
	pub unk1: u8,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextNode {
	pub text_id: u32,
	pub color: u32,
	pub alignment: u16,
	pub font: FontType,
	pub font_size: u8,
	pub edge_color: u32,
	
	#[br(temp)]
	#[bw(calc = (*bold as u8) << 7 | (*italic as u8) << 6 | (*edge as u8) << 5 | (*glare as u8) << 4 | (*multiline as u8) << 3 | (*ellipsis as u8) << 2 | (*paragraph as u8) << 1 | (*emboss as u8))]
	field: u8,
	#[br(calc = (field & 0x80) == 0x80)]
	#[bw(ignore)]
	pub bold: bool,
	#[br(calc = (field & 0x40) == 0x40)]
	#[bw(ignore)]
	pub italic: bool,
	#[br(calc = (field & 0x20) == 0x20)]
	#[bw(ignore)]
	pub edge: bool,
	#[br(calc = (field & 0x10) == 0x10)]
	#[bw(ignore)]
	pub glare: bool,
	#[br(calc = (field & 0x08) == 0x08)]
	#[bw(ignore)]
	pub multiline: bool,
	#[br(calc = (field & 0x04) == 0x04)]
	#[bw(ignore)]
	pub ellipsis: bool,
	#[br(calc = (field & 0x02) == 0x02)]
	#[bw(ignore)]
	pub paragraph: bool,
	#[br(calc = (field & 0x01) == 0x01)]
	#[bw(ignore)]
	pub emboss: bool,
	
	pub sheet_type: SheetType,
	pub char_spacing: u8,
	pub line_spacing: u8,
	pub unk2: u32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NineGridNode {
	pub part_list_id: u32,
	pub part_id: u32,
	pub grid_parts_type: GridPartsType,
	pub grid_render_type: GridRenderType,
	pub top_offset: i16,
	pub bottom_offset: i16,
	pub left_offset: i16,
	pub right_offset: i16,
	pub unk1: u16,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CounterNode {
	pub part_list_id: u32,
	pub part_id: u8,
	pub number_width: u8,
	pub comma_width: u8,
	pub space_width: u8,
	pub alignment: u16,
	pub unk1: u16,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CollisionNode {
	pub collision_type: CollisionType,
	pub unk1: u16,
	pub x: i32,
	pub y: i32,
	pub radius: u32,
}

#[binrw]
#[brw(little)]
#[br(import(parent_type: super::ComponentType, node_type: u32))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComponentNode {
	pub index: u8,
	pub up: u8,
	pub down: u8,
	pub left: u8,
	pub right: u8,
	pub cursor: u8,
	
	#[br(temp)]
	#[bw(calc = (*repeat_up as u8) << 7 | (*repeat_down as u8) << 6 | (*repeat_left as u8) << 5 | (*repeat_right as u8) << 4 | (*unk1 as u8) << 3 | (*unk2 as u8) << 2 | (*unk3 as u8) << 1 | (*unk4 as u8))]
	field: u8,
	#[br(calc = (field & 0x80) == 0x80)]
	#[bw(ignore)]
	pub repeat_up: bool,
	#[br(calc = (field & 0x40) == 0x40)]
	#[bw(ignore)]
	pub repeat_down: bool,
	#[br(calc = (field & 0x20) == 0x20)]
	#[bw(ignore)]
	pub repeat_left: bool,
	#[br(calc = (field & 0x10) == 0x10)]
	#[bw(ignore)]
	pub repeat_right: bool,
	#[br(calc = (field & 0x08) == 0x08)]
	#[bw(ignore)]
	pub unk1: bool,
	#[br(calc = (field & 0x04) == 0x04)]
	#[bw(ignore)]
	pub unk2: bool,
	#[br(calc = (field & 0x02) == 0x02)]
	#[bw(ignore)]
	pub unk3: bool,
	#[br(calc = (field & 0x01) == 0x01)]
	#[bw(ignore)]
	pub unk4: bool,
	
	pub unk5: u8,
	pub offset_x: i16,
	pub offset_y: i16,
	#[br(args(parent_type,))]
	pub component_node_data: ComponentNodeNode,
	#[br(calc(node_type))]
	#[bw(ignore)]
	pub component_id: u32,
}

// ---------------------------------------- //

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ButtonComponentNode {
	pub text_id: u32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WindowComponentNode {
	pub title_text_id: u32,
	pub subtitle_text_id: u32,
	#[br(map = |v: u8| v != 0)]
	#[bw(map = |v: &bool| if *v {1u8} else {0})]
	pub close_button: bool,
	#[br(map = |v: u8| v != 0)]
	#[bw(map = |v: &bool| if *v {1u8} else {0})]
	pub config_button: bool,
	#[br(map = |v: u8| v != 0)]
	#[bw(map = |v: &bool| if *v {1u8} else {0})]
	pub help_button: bool,
	#[br(map = |v: u8| v != 0)]
	#[bw(map = |v: &bool| if *v {1u8} else {0})]
	pub header: bool,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CheckBoxComponentNode {
	pub text_id: u32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RadioButtonComponentNode {
	pub text_id: u32,
	pub group_id: u32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GaugeComponentNode {
	pub indicator: i32,
	pub min: i32,
	pub max: i32,
	pub value: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SliderComponentNode {
	pub min: i32,
	pub max: i32,
	pub step: i32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextInputComponentNode {
	pub max_width: u32,
	pub max_line: u32,
	pub max_s_byte: u32,
	pub max_char: u32,
	
	#[br(temp)]
	#[bw(calc = (*capitalize as u8) << 7 | (*mask as u8) << 6 | (*auto_translate_enabled as u8) << 5 | (*history_enabled as u8) << 4 | (*ime_enabled as u8) << 3 | (*escape_clears as u8) << 2 | (*caps_allowed as u8) << 1 | (*lower_allowed as u8))]
	field: u8,
	#[br(calc = (field & 0x80) == 0x80)]
	#[bw(ignore)]
	pub capitalize: bool,
	#[br(calc = (field & 0x40) == 0x40)]
	#[bw(ignore)]
	pub mask: bool,
	#[br(calc = (field & 0x20) == 0x20)]
	#[bw(ignore)]
	pub auto_translate_enabled: bool,
	#[br(calc = (field & 0x10) == 0x10)]
	#[bw(ignore)]
	pub history_enabled: bool,
	#[br(calc = (field & 0x08) == 0x08)]
	#[bw(ignore)]
	pub ime_enabled: bool,
	#[br(calc = (field & 0x04) == 0x04)]
	#[bw(ignore)]
	pub escape_clears: bool,
	#[br(calc = (field & 0x02) == 0x02)]
	#[bw(ignore)]
	pub caps_allowed: bool,
	#[br(calc = (field & 0x01) == 0x01)]
	#[bw(ignore)]
	pub lower_allowed: bool,
	
	#[br(temp)]
	#[bw(calc = (*numbers_allowed as u8) << 7 | (*symbols_allowed as u8) << 6 | (*word_wrap as u8) << 5 | (*multiline as u8) << 4 | (*auto_max_width as u8) << 3 | (*unk1 as u8))]
	field2: u8,
	#[br(calc = (field2 & 0x80) == 0x80)]
	#[bw(ignore)]
	pub numbers_allowed: bool,
	#[br(calc = (field2 & 0x40) == 0x40)]
	#[bw(ignore)]
	pub symbols_allowed: bool,
	#[br(calc = (field2 & 0x20) == 0x20)]
	#[bw(ignore)]
	pub word_wrap: bool,
	#[br(calc = (field2 & 0x10) == 0x10)]
	#[bw(ignore)]
	pub multiline: bool,
	#[br(calc = (field2 & 0x08) == 0x08)]
	#[bw(ignore)]
	pub auto_max_width: bool,
	#[br(calc = field2 & 0x07)]
	#[bw(ignore)]
	pub unk1: u8,
	
	pub charset: u16,
	pub charset_extras: [u16; 8],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NumericInputComponentNode {
	pub value: i32,
	pub max: i32,
	pub min: i32,
	pub add: i32,
	pub unk1: u32,
	#[br(map = |v: u8| v != 0)]
	#[bw(map = |v: &bool| if *v {1u8} else {0})]
	pub comma: bool,
	pub unk2: [u8; 3],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListComponentNode {
	pub row_num: u16,
	pub column_num: u16,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TabbedComponentNode {
	pub text_id: u32,
	pub group_id: u32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListItemComponentNode {
	#[br(map = |v: u8| v != 0)]
	#[bw(map = |v: &bool| if *v {1u8} else {0})]
	pub toggle: bool,
	pub unk1: [u8; 3],
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NineGridTextComponentNode {
	pub text_id: u32,
}