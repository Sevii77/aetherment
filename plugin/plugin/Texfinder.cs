using System;
using System.Numerics;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using ImGuiNET;
using FFXIVClientStructs.FFXIV.Component.GUI;

namespace Aetherment;

public partial class TexFinder {
	public bool shouldDraw = false;
	private Vector2 cursorPos;
	private List<IntPtr> nodes;
	private bool locked = false;
	private bool lastheld = false;
	private int selected = 0;
	private bool hr1 = true;
	
	public unsafe TexFinder() {
		nodes = new();
	}
	
	public void OpenConf() {shouldDraw = true;}
	
	public void Draw() {
		if(!shouldDraw)
			return;
		
		if(ImGui.GetIO().KeyCtrl && ImGui.GetIO().KeyShift) {
			if(!lastheld) {
				locked = !locked;
				lastheld = true;
			}
		} else
			lastheld = false;
		
		if(!locked) {
			selected = 0;
			CheckElements();
		}
		
		var padding = ImGui.GetStyle().FramePadding;
		
		ImGui.SetNextWindowSize(new Vector2(500, 400), ImGuiCond.FirstUseEver);
		ImGui.Begin("Texture Finder", ref shouldDraw);
		
		ImGui.Checkbox("High res", ref hr1);
		ImGui.Text($"{(locked ? "Locked" : "Unlocked")} (Shift + Ctrl to toggle)");
		
		if(nodes.Count > 0) {
			unsafe {
				for(int i = 0; i < nodes.Count; i++)
					if(i != selected)
						DrawNode((AtkResNode*)nodes[i], 0xFFFF0000);
				
				var node = (AtkImageNode*)nodes[selected];
				var n = ((AtkResNode*)nodes[selected]);
				DrawNode(n, 0xFF00FF00);
				
				// if(ImGuiAeth.ButtonIcon(FontAwesomeIcon.ArrowLeft))
				if(ImGui.Button("<"))
					selected = Math.Max(selected - 1, 0);
				ImGui.SameLine();
				// ImGui.SetNextItemWidth(ImGui.CalcTextSize($"{nodes.Count - 1}").X + ImGuiAeth.PaddingX * 2);
				ImGui.SetNextItemWidth(ImGui.CalcTextSize($"{nodes.Count - 1}").X + padding.X * 2);
				if(ImGui.InputInt("##selected", ref selected, 0, 0))
					selected = Math.Clamp(selected, 0, nodes.Count - 1);
				ImGui.SameLine();
				ImGui.Text($"/");
				ImGui.SameLine();
				ImGui.Text($"{nodes.Count - 1}");
				ImGui.SameLine();
				// if(ImGuiAeth.ButtonIcon(FontAwesomeIcon.ArrowRight))
				if(ImGui.Button(">"))
					selected = Math.Min(selected + 1, nodes.Count - 1);
				
				try {
					var draw = ImGui.GetWindowDrawList();
					
					if(n->Type == NodeType.Image) {
						var part = node->PartsList->Parts[node->PartId];
						var type = part.UldAsset->AtkTexture.TextureType;
						var tex = part.UldAsset->AtkTexture;
						var texture = type == TextureType.Resource ?
							tex.Resource->KernelTextureObject :
							tex.KernelTexture;
						
						// AtkTextureResource.Unk_1 seems to be int iconid with 1000000 added if hq
						// TODO: make pr to ffxivclientstructs to rename that
						if(type == TextureType.Resource) {
							// var path = PenumbraApi.GetGamePath(tex.Resource->TexFileResourceHandle->ResourceHandle.FileName.ToString());
							var path = tex.Resource->TexFileResourceHandle->ResourceHandle.FileName.ToString();
							ImGui.SameLine();
							if(ImGui.Button(path, new Vector2(0, ImGui.GetFontSize() + padding.Y * 2)))
								ImGui.SetClipboardText(path);
							if(ImGui.IsItemHovered())
								ImGui.SetTooltip("Copy to clipboard");
						}
						
						// The preview
						var s = ImGui.GetContentRegionAvail();
						var ratio = Math.Min(s.X / texture->Width, s.Y / texture->Height);
						var pos = ImGui.GetCursorScreenPos();
						var imgsize = new Vector2(texture->Width, texture->Height) * ratio;
						draw.AddImage(new IntPtr(texture->D3D11ShaderResourceView), pos, pos + imgsize);
						
						var scale = hr1 ? 2 : 1;
						var (u, v) = (part.U * scale, part.V * scale);
						var (w, h) = (part.Width * scale, part.Height * scale);
						
						pos += new Vector2(u, v) * ratio;
						draw.AddRect(pos, pos + new Vector2(w, h) * ratio, 0xFF00FF00);
					} else {
						var ninegrid = (AtkNineGridNode*)nodes[selected];
						ImGui.Text($"This element {((ninegrid->PartsTypeRenderType & 2) == 2 ? "tiles" : "stretches")} to fill");
						var partcount = (ninegrid->PartsTypeRenderType & 1) == 1 ? 9 : 1;
						var size = partcount == 9 ? ImGui.GetContentRegionAvail() / 3 - padding * 2 : ImGui.GetContentRegionAvail();
						
						for(uint j = 0; j < partcount; j++) {
							if(j % 3 != 0)
								ImGui.SameLine();
							
							var i = j + ninegrid->PartID;
							
							ImGui.BeginChild($"{i}9grid", size);
							var part = node->PartsList->Parts[i];
							var type = part.UldAsset->AtkTexture.TextureType;
							var tex = part.UldAsset->AtkTexture;
							var texture = type == TextureType.Resource ?
								tex.Resource->KernelTextureObject :
								tex.KernelTexture;
							
							if(type == TextureType.Resource) {
								// var path = PenumbraApi.GetGamePath(tex.Resource->TexFileResourceHandle->ResourceHandle.FileName.ToString());
								var path = tex.Resource->TexFileResourceHandle->ResourceHandle.FileName.ToString();
								if(ImGui.Button(path, new Vector2(0, ImGui.GetFontSize() + padding.Y * 2)))
									ImGui.SetClipboardText(path);
								if(ImGui.IsItemHovered())
									ImGui.SetTooltip("Copy to clipboard");
							}
							
							var s = ImGui.GetContentRegionAvail();
							var ratio = Math.Min(s.X / texture->Width, s.Y / texture->Height);
							var pos = ImGui.GetCursorScreenPos();
							var imgsize = new Vector2(texture->Width, texture->Height) * ratio;
							draw.AddImage(new IntPtr(texture->D3D11ShaderResourceView), pos, pos + imgsize);
							
							// outline
							var scale = hr1 ? 2 : 1;
							var (u, v) = (part.U * scale, part.V * scale);
							var (w, h) = (part.Width * scale * ratio, part.Height * scale * ratio);
							
							pos += new Vector2(u, v) * ratio;
							draw.AddRect(pos, pos + new Vector2(w, h), 0xFF00FF00);
							
							// grid if divide type
							if(partcount == 1) {
								var rs = scale * ratio;
								draw.AddLine(pos + new Vector2(0, ninegrid->TopOffset * rs), pos + new Vector2(w, ninegrid->TopOffset * rs), 0xFF00FF00);
								draw.AddLine(pos + new Vector2(0, h - ninegrid->BottomOffset * rs), pos + new Vector2(w, h - ninegrid->BottomOffset * rs), 0xFF00FF00);
								draw.AddLine(pos + new Vector2(ninegrid->LeftOffset * rs, 0), pos + new Vector2(ninegrid->LeftOffset * rs, h), 0xFF00FF00);
								draw.AddLine(pos + new Vector2(w - ninegrid->RightOffset * rs, 0), pos + new Vector2(w - ninegrid->RightOffset * rs, h), 0xFF00FF00);
							}
							
							ImGui.EndChild();
						}
					}
				} catch {
					ImGui.Text("Invalid element");
				}
			}
		}
		
		ImGui.End();
	}
	
	private unsafe void DrawNode(AtkResNode* node, uint clr) {
		if(node == null)
			return;
		
		var (pos, scale) = GlobalNode(node);
		var width = node->Width * scale.X;
		var height = node->Height * scale.Y;
		
		// TODO: rotation
		ImGui.GetForegroundDrawList().AddRect(pos, pos + new Vector2(width, height), clr);
	}
	
	private void CheckElements() {
		cursorPos = ImGui.GetMousePos();
		
		unsafe {
			nodes.Clear();
			
			var layersAddress = (IntPtr)AtkStage.GetSingleton()->RaptureAtkUnitManager;
			for(var layerI = 12; layerI >= 0; layerI--) {
				var layer = Marshal.PtrToStructure<AtkUnitList>(layersAddress + 0x30 + 0x810 * layerI);
				
				for(var atkI = 0; atkI < layer.Count; atkI++) {
					// var atk = (AtkUnitBase*)(&layer.Entries)[atkI];
					var atk = layer.EntriesSpan[atkI].Value;
					if(atk->IsVisible)
						CheckNodes(atk->UldManager, ref nodes);
				}
			}
			
			// this shouldnt be needed but stuff like icons on maps dont work and idfk why
			nodes.Sort((x, y) => {
				var a = (AtkResNode*)x;
				var b = (AtkResNode*)y;
				
				var scaleA = GlobalNode(a).Item2;
				var scaleB = GlobalNode(b).Item2;
				
				return (a->Width * a->Height * scaleA.X * scaleA.Y).CompareTo(b->Width * b->Height * scaleB.X * scaleB.Y);
			});
		}
	}
	
	private unsafe void CheckNodes(AtkUldManager uld, ref List<IntPtr> elements) {
		for(var nodeI = (int)uld.NodeListCount - 1; nodeI >= 0; nodeI--) {
			var node = uld.NodeList[nodeI];
			if(!GlobalNodeVisible(node))
				continue;
			
			if(node->Type == NodeType.Image || node->Type == NodeType.NineGrid) {
				var imgnode = (AtkImageNode*)node;
				var type = imgnode->PartsList->Parts[imgnode->PartId].UldAsset->AtkTexture.TextureType;
				if(type == TextureType.Resource || type == TextureType.KernelTexture) {
					var (pos, scale) = GlobalNode(node);
					var width = node->Width * scale.X;
					var height = node->Height * scale.Y;
					
					if(cursorPos.X > pos.X && cursorPos.X < pos.X + width && cursorPos.Y > pos.Y && cursorPos.Y < pos.Y + height)
						elements.Add((IntPtr)node);
				}
			} else if((ushort)node->Type >= 1000) {
				CheckNodes(((AtkComponentNode*)node)->Component->UldManager, ref elements);
			}
		}
	}
	
	private unsafe bool GlobalNodeVisible(AtkResNode* node) {
		while(node != null) {
			if(!node->IsVisible)
				return false;
			
			node = node->ParentNode;
		}
		
		return true;
	}
	
	private unsafe (Vector2, Vector2) GlobalNode(AtkResNode* node) {
		var pos = Vector2.Zero;
		var scale = Vector2.One;
		
		while(node != null) {
			var s = new Vector2(node->ScaleX, node->ScaleY);
			scale *= s;
			pos *= s;
			pos += new Vector2(node->X, node->Y);
			
			node = node->ParentNode;
		}
		
		return (pos, scale);
	}
}