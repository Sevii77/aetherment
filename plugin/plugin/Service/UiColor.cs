using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using Dalamud.Hooking;

namespace Aetherment;

public class UiColor: IDisposable {
	[StructLayout(LayoutKind.Sequential, Pack = 1)]
	public struct Color {
		public byte useTheme;
		public uint index;
		public uint clr;
	}
	
	private Dictionary<(bool, uint), uint> colors;
	
	public unsafe UiColor() {
		colors = new();
		
		UiColorHandlerHook = Aetherment.HookProv.HookFromAddress<UiColorHandlerDelegate>(Aetherment.SigScanner.ScanText("4C 8B 91 ?? ?? ?? ?? 4C 8B D9 49 8B 02"), UiColorHandler);
		UiColorHandlerHook.Enable();
		
		setUiColors = SetUiColors;
	}
	
	public void Dispose() {
		UiColorHandlerHook.Dispose();
	}
	
	private unsafe delegate uint UiColorHandlerDelegate(nint self, byte use_theme, uint index);
	private Hook<UiColorHandlerDelegate> UiColorHandlerHook;
	private uint UiColorHandler(nint self, byte use_theme, uint index) {
		lock(colors) {
			if(colors.TryGetValue((use_theme != 0, index), out var color))
				return color;
		}
		
		return UiColorHandlerHook.Original(self, use_theme, index);
	}
	
	public SetUiColorsDelegate setUiColors;
	public unsafe delegate void SetUiColorsDelegate(Color* array, nint length);
	public unsafe void SetUiColors(Color* array, nint length) {
		lock(colors) {
			colors.Clear();
			for(int i = 0; i < length; i++) {
				var color = array[i];
				colors.Add((color.useTheme != 0, color.index), color.clr);
			}
		}
	}
}