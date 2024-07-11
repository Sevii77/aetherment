using System;
using System.Runtime.InteropServices;
using Dalamud.Game.Command;
using Dalamud.IoC;
using Dalamud.Plugin;
using Dalamud.Plugin.Services;

namespace Aetherment;

public class Aetherment : IDalamudPlugin {
	public string Name => "Aetherment";
	
	[PluginService] public static IDalamudPluginInterface Interface {get; private set;} = null!;
	[PluginService] public static ICommandManager         Commands  {get; private set;} = null!;
	[PluginService] public static IPluginLog              Logger    {get; private set;} = null!;
	[PluginService] public static IObjectTable            Objects   {get; private set;} = null!;
	// [PluginService][RequiredVersion("1.0")] public static TitleScreenMenu        TitleMenu  {get; private set;} = null!;
	
	private const string maincommand = "/aetherment";
	private const string texfindercommand = "/texfinder";
	
	private static IntPtr state;
	
	private Penumbra penumbra;
	private TexFinder texfinder;
	
	[StructLayout(LayoutKind.Sequential)]
	private unsafe struct Initializers {
		public IntPtr log;
		public PenumbraFunctions penumbra;
	}
	
	[StructLayout(LayoutKind.Sequential)]
	private unsafe struct PenumbraFunctions {
		public FFI.Str config_dir;
		public IntPtr redraw;
		public IntPtr redraw_self;
		public IntPtr root_path;
		public IntPtr mod_list;
		public IntPtr add_mod_entry;
		public IntPtr reload_mod;
		public IntPtr set_mod_enabled;
		public IntPtr set_mod_priority;
		public IntPtr set_mod_inherit;
		public IntPtr set_mod_settings;
		public IntPtr get_mod_settings;
		public IntPtr current_collection;
		public IntPtr get_collections;
	}
	
	public unsafe Aetherment() {
		log = Log;
		penumbra = new();
		texfinder = new();
		
		var init = new Initializers {
			log = Marshal.GetFunctionPointerForDelegate(log),
			penumbra = new PenumbraFunctions {
				config_dir = Interface.ConfigDirectory.Parent! + "/Penumbra/",
				redraw = Marshal.GetFunctionPointerForDelegate(penumbra.redraw),
				redraw_self = Marshal.GetFunctionPointerForDelegate(penumbra.redrawSelf),
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
				current_collection = Marshal.GetFunctionPointerForDelegate(penumbra.currentCollection),
				get_collections = Marshal.GetFunctionPointerForDelegate(penumbra.getCollections),
			},
		};
		
		state = initialize(init);
		
		Interface.UiBuilder.Draw += Draw;
		Interface.UiBuilder.OpenMainUi += OpenConf;
		
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
		destroy(state);
		Interface.UiBuilder.Draw -= Draw;
		Interface.UiBuilder.OpenMainUi -= OpenConf;
		Commands.RemoveHandler(maincommand);
		Commands.RemoveHandler(texfindercommand);
		// if(watcher != null)
		// 	watcher.Dispose();
		state = IntPtr.Zero;
	}
	
	private void OpenConf() {
		OnCommand(maincommand, "");
	}
	
	private void Draw() {
		try {
			draw(state);
		} catch {
			// Kill("Fatal error in rendering", 0);
		}
		
		texfinder.Draw();
	}
	
	private void OnCommand(string cmd, string args) {
		if(cmd == texfindercommand) {
			texfinder.shouldDraw = !texfinder.shouldDraw;
		}
		
		if(cmd != maincommand)
			return;
		
		command(state, args);
	}
	
	private LogDelegate log;
	private unsafe delegate void LogDelegate(byte mode, FFI.Str str);
	private unsafe void Log(byte mode, FFI.Str str) {
		if(mode == 255) {
			// Kill(str, 2);
			Logger.Debug("TODO: kill plugin");
			Logger.Error(str);
		} else if(mode == 1)
			Logger.Error(str);
		else
			Logger.Debug(str);
	}
	
	[DllImport("aetherment_core.dll")] private static extern unsafe IntPtr initialize(Initializers data);
	[DllImport("aetherment_core.dll")] private static extern unsafe void destroy(IntPtr state);
	[DllImport("aetherment_core.dll")] private static extern unsafe void command(IntPtr state, FFI.Str args);
	[DllImport("aetherment_core.dll")] private static extern unsafe void draw(IntPtr state);
}