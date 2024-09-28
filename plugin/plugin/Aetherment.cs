using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.InteropServices;
using Dalamud.Game.Command;
using Dalamud.IoC;
using Dalamud.Plugin;
using Dalamud.Plugin.Services;

namespace Aetherment;

public class Aetherment: IDalamudPlugin {
	public string Name => "Aetherment";
	
	[PluginService] public static IDalamudPluginInterface Interface {get; private set;} = null!;
	[PluginService] public static ICommandManager         Commands  {get; private set;} = null!;
	[PluginService] public static IPluginLog              Logger    {get; private set;} = null!;
	[PluginService] public static IObjectTable            Objects   {get; private set;} = null!;
	[PluginService] public static ITitleScreenMenu        TitleMenu {get; private set;} = null!;
	[PluginService] public static ITextureProvider        Textures  {get; private set;} = null!;
	
	private const string maincommand = "/aetherment";
	private const string texfindercommand = "/texfinder";
	
	internal static nint state;
	private static string? error;
	private static Dalamud.Interface.IReadOnlyTitleScreenMenuEntry? titleEntry;
	
	private Issue issue;
	private Penumbra penumbra;
	private DalamudStyle dalamud;
	private TexFinder texfinder;
	
	[StructLayout(LayoutKind.Sequential)]
	private unsafe struct Initializers {
		public nint log;
		public IssueFunctions issue;
		public PenumbraFunctions penumbra;
		public nint dalamud_add_style;
	}
	
	[StructLayout(LayoutKind.Sequential)]
	private unsafe struct IssueFunctions {
		public nint ui_resolution;
		public nint ui_theme;
	}

	[StructLayout(LayoutKind.Sequential)]
	private unsafe struct PenumbraFunctions {
		// public FFI.Str config_dir;
		public nint redraw;
		public nint redraw_self;
		public nint is_enabled;
		public nint root_path;
		public nint mod_list;
		public nint add_mod_entry;
		public nint reload_mod;
		public nint set_mod_enabled;
		public nint set_mod_priority;
		public nint set_mod_inherit;
		public nint set_mod_settings;
		public nint get_mod_settings;
		public nint get_collection;
		public nint get_collections;
	}
	
	public unsafe Aetherment() {
		// var c = FFXIVClientStructs.FFXIV.Client.System.Framework.Framework.Instance()->SystemConfig.SystemConfigBase.ConfigBase;
		// for(var i = 0; i < c.ConfigCount; ++i) {
		// 	var s = (nint)c.ConfigEntry[i].Name == 0 ? "Invalid Name\0" : System.Text.Encoding.UTF8.GetString(c.ConfigEntry[i].Name, 128);
		// 	s = s.Substring(0, s.IndexOf('\0'));
		// 	if(s.ToLowerInvariant().Contains("resolution") || s.ToLowerInvariant().Contains("ui") || s.ToLowerInvariant().Contains("theme"))
		// 		Logger.Debug($"[{i}] {s}: {c.ConfigEntry[i].Value.UInt}");
		// }
		
		log = Log;
		issue = new();
		penumbra = new();
		dalamud = new();
		texfinder = new();
		
		var init = new Initializers {
			log = Marshal.GetFunctionPointerForDelegate(log),
			issue = new IssueFunctions {
				ui_resolution = Marshal.GetFunctionPointerForDelegate(issue.getUiResolution),
				ui_theme = Marshal.GetFunctionPointerForDelegate(issue.getUiTheme),
			},
			penumbra = new PenumbraFunctions {
				// config_dir = Interface.ConfigDirectory.Parent! + "/Penumbra/",
				redraw = Marshal.GetFunctionPointerForDelegate(penumbra.redraw),
				redraw_self = Marshal.GetFunctionPointerForDelegate(penumbra.redrawSelf),
				is_enabled = Marshal.GetFunctionPointerForDelegate(penumbra.isEnabled),
				root_path = Marshal.GetFunctionPointerForDelegate(penumbra.rootPath),
				mod_list = Marshal.GetFunctionPointerForDelegate(penumbra.modList),
				add_mod_entry = Marshal.GetFunctionPointerForDelegate(penumbra.addModEntry),
				reload_mod = Marshal.GetFunctionPointerForDelegate(penumbra.reloadMod),
				set_mod_enabled = Marshal.GetFunctionPointerForDelegate(penumbra.setModEnabled),
				set_mod_priority = Marshal.GetFunctionPointerForDelegate(penumbra.setModPriority),
				set_mod_inherit = Marshal.GetFunctionPointerForDelegate(penumbra.setModInherit),
				set_mod_settings = Marshal.GetFunctionPointerForDelegate(penumbra.setModSettings),
				get_mod_settings = Marshal.GetFunctionPointerForDelegate(penumbra.getModSettings),
				// default_collection = Marshal.GetFunctionPointerForDelegate(penumbra.defaultCollection),
				get_collection = Marshal.GetFunctionPointerForDelegate(penumbra.getCollection),
				get_collections = Marshal.GetFunctionPointerForDelegate(penumbra.getCollections),
			},
			dalamud_add_style = Marshal.GetFunctionPointerForDelegate(dalamud.addStyle),
		};
		
		try {
			state = initialize(init);
		} catch(Exception e) {
			Kill(e.ToString(), 2);
		}
		
		Interface.UiBuilder.Draw += Draw;
		Interface.UiBuilder.OpenMainUi += OpenConf;
		
		// TODO: proper icon
		var icon_data = new byte[64 * 64 * 4];
		var icon = Textures.CreateFromRaw(new Dalamud.Interface.Textures.RawImageSpecification(64, 64, 28), icon_data, "Aetherment Icon");
		titleEntry ??= TitleMenu.AddEntry(1, "Manage Aetherment", icon, OpenConf);
		
		Commands.AddHandler(maincommand, new CommandInfo(OnCommand) {
			HelpMessage = "Open Aetherment menu"
		});
		Commands.AddHandler(texfindercommand, new CommandInfo(OnCommand) {
			HelpMessage = "Open the Texture Finder tool"
		});
		
		// Reload if the rust part changes
		// if(Interface.IsDev) {
		// 	watcher = new FileSystemWatcher($"{Interface.AssemblyLocation.DirectoryName}", "aetherment_core.dll");
		// 	watcher.NotifyFilter = NotifyFilters.LastWrite;
		// 	watcher.Changed += (object _, FileSystemEventArgs e) => {
		// 		watcher.EnableRaisingEvents = false;
		// 		Task.Run(() => {
		// 			Task.Delay(1000);
		// 			ReloadPlugin();
		// 		});
		// 	};
		// 	watcher.EnableRaisingEvents = true;
		// }
	}
	
	public void Dispose() {
		if(state != 0)
			destroy(state);
		
		FFI.Str.Drop();
		
		Interface.UiBuilder.Draw -= Draw;
		Interface.UiBuilder.OpenMainUi -= OpenConf;
		
		if(titleEntry != null) {
			TitleMenu.RemoveEntry(titleEntry);
			titleEntry = null;
		}
		
		Commands.RemoveHandler(maincommand);
		Commands.RemoveHandler(texfindercommand);
		
		// if(watcher != null)
		// 	watcher.Dispose();
		state = 0;
	}
	
	private void OpenConf() {
		OnCommand(maincommand, "");
	}
	
	private void Draw() {
		FFI.Str.HandleResources();
		
		if(state != 0) {
			try {
				draw(state);
			} catch {
				Kill("Fatal error in draw", 1);
			}
		} else {
			ImGuiNET.ImGui.Begin("Aetherment");
			ImGuiNET.ImGui.Text("Aetherment has encountered an unrecoverable error");
			ImGuiNET.ImGui.Text(error ?? "No Error");
			ImGuiNET.ImGui.End();
		}
		
		texfinder.Draw();
	}
	
	private void OnCommand(string cmd, string args) {
		if(cmd == texfindercommand) {
			texfinder.shouldDraw = !texfinder.shouldDraw;
			return;
		}
		
		if(state == 0)
			return;
		
		if(cmd != maincommand)
			return;
		
		command(state, args);
	}
	
	private void Kill(string msg, byte strip) {
		if(error != null)
			return;
		
		var frames = new StackTrace(true).GetFrames();
		var stack = new List<string>();
		stack.Add($"{msg.Replace("\n", "\n\t")}");
		
		for(int i = strip; i < frames.Length; i++)
			// we dont care about the stack produced by ffi functions themselves
			// or by functions outside our own assembly
			if(frames[i].GetFileLineNumber() > 0 && frames[i].GetMethod()?.Module == typeof(Aetherment).Module)
				stack.Add($"at {frames[i].GetMethod()} {frames[i].GetFileName()}:{frames[i].GetFileLineNumber()}:{frames[i].GetFileColumnNumber()}");
		
		error = string.Join("\n", stack);
		state = 0;
		
		Logger.Fatal(error);
	}
	
	private LogDelegate log;
	private unsafe delegate void LogDelegate(byte mode, FFI.Str str);
	private unsafe void Log(byte mode, FFI.Str str) {
		if(mode == 255) {
			// Logger.Debug("TODO: kill plugin");
			// Logger.Error(str);
			Kill(str, 2);
		} else if(mode == 1)
			Logger.Error(str);
		else
			Logger.Debug(str);
	}
	
	[DllImport("aetherment_core.dll")] private static extern unsafe nint initialize(Initializers data);
	[DllImport("aetherment_core.dll")] private static extern unsafe void destroy(nint state);
	[DllImport("aetherment_core.dll")] private static extern unsafe void command(nint state, FFI.Str args);
	[DllImport("aetherment_core.dll")] private static extern unsafe void draw(nint state);
}