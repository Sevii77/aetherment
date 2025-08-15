using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Diagnostics.CodeAnalysis;
using System.Numerics;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using Dalamud.Game;
using Dalamud.Game.Command;
using Dalamud.Interface.Textures.TextureWraps;
using Dalamud.Interface.ImGuiNotification;
using Dalamud.IoC;
using Dalamud.Plugin;
using Dalamud.Plugin.Services;
using Dalamud.Bindings.ImGui;

namespace Aetherment;

public class Aetherment: IDalamudPlugin {
	public string Name => "Aetherment";
	
	[PluginService] public static IDalamudPluginInterface Interface  {get; private set;} = null!;
	[PluginService] public static ICommandManager         Commands   {get; private set;} = null!;
	[PluginService] public static IPluginLog              Logger     {get; private set;} = null!;
	[PluginService] public static IObjectTable            Objects    {get; private set;} = null!;
	[PluginService] public static ITitleScreenMenu        TitleMenu  {get; private set;} = null!;
	[PluginService] public static ITextureProvider        Textures   {get; private set;} = null!;
	[PluginService] public static ISigScanner             SigScanner {get; private set;} = null!;
	[PluginService] public static IGameInteropProvider    HookProv   {get; private set;} = null!;
	[PluginService] public static INotificationManager    NotifMan   {get; private set;} = null!;
	
	private const string maincommand = "/aetherment";
	private const string texfindercommand = "/texfinder";
	
	private bool open = false;
	private bool reset_cursor = false;
	
	internal static nint state;
	private static string? error;
	private static Dalamud.Interface.IReadOnlyTitleScreenMenuEntry? title_entry;
	private IActiveNotification? notification;

	// idfc, entry changed to some other bs that always returns a texture wrap but cant be provided a texture wrap.
	// i'm not going to dive into the docs to figure out a "proper way"
	private struct TextureWrap: Dalamud.Interface.Textures.ISharedImmediateTexture {
		private IDalamudTextureWrap wrap;
		
		public TextureWrap(IDalamudTextureWrap wrap) {
			this.wrap = wrap;
		}
		
		public IDalamudTextureWrap? GetWrapOrDefault(IDalamudTextureWrap? defaultWrap = null) {
			return wrap ?? defaultWrap;
		}
		
		public IDalamudTextureWrap GetWrapOrEmpty() {
			return wrap;
		}
		
		public Task<IDalamudTextureWrap> RentAsync(System.Threading.CancellationToken cancellationToken = default) {
			var wrap = this.wrap;
			return Task.Run(() => {return wrap;});
		}
		
		public bool TryGetWrap([NotNullWhen(true)] out IDalamudTextureWrap? texture, out Exception? exception) {
			texture = wrap;
			exception = null;
			return true;
		}
	}
	
	private Requirement requirement;
	private Penumbra penumbra;
	private DalamudStyle dalamud;
	private UiColor uicolor;
	private TexFinder texfinder;
	
	private nint device;
	
	[StructLayout(LayoutKind.Sequential)]
	public unsafe struct Initializers {
		public nint ffi_str_drop;
		public nint log;
		public nint setNotification;
		public RequirementFunctions requirement;
		public PenumbraFunctions penumbra;
		public ServicesFunctions services;
		public nint d3d11_device;
	}
	
	[StructLayout(LayoutKind.Sequential)]
	public unsafe struct RequirementFunctions {
		public nint ui_resolution;
		public nint ui_theme;
	}
	
	[StructLayout(LayoutKind.Sequential)]
	public unsafe struct ServicesFunctions {
		public nint set_ui_colors;
		public nint dalamud_add_style;
	}

	[StructLayout(LayoutKind.Sequential)]
	public unsafe struct PenumbraFunctions {
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
	
	[StructLayout(LayoutKind.Sequential)]
	public unsafe struct Io {
		public float width;
		public float height;
		public float mouse_x;
		public float mouse_y;
		public float wheel_x;
		public float wheel_y;
		public uint mods;
		public nint input_buf_ptr;
		public nint input_buf_len;
		public float ui_scale;
		public byte* set_keyboard_focus;
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
		setNotification = SetNotification;
		requirement = new();
		penumbra = new();
		dalamud = new();
		uicolor = new();
		texfinder = new();
		
		device = Interface.UiBuilder.DeviceHandle;
		
		var init = new Initializers {
			ffi_str_drop = Marshal.GetFunctionPointerForDelegate(FFI.Str.drop),
			log = Marshal.GetFunctionPointerForDelegate(log),
			setNotification = Marshal.GetFunctionPointerForDelegate(setNotification),
			requirement = new RequirementFunctions {
				ui_resolution = Marshal.GetFunctionPointerForDelegate(requirement.getUiResolution),
				ui_theme = Marshal.GetFunctionPointerForDelegate(requirement.getUiTheme),
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
			services = new ServicesFunctions {
				set_ui_colors = Marshal.GetFunctionPointerForDelegate(uicolor.setUiColors),
				dalamud_add_style = Marshal.GetFunctionPointerForDelegate(dalamud.addStyle),
			},
			d3d11_device = device,
		};
		
		try {
			state = Native.initialize(init);
			open = Native.config_plugin_open_on_launch(state) != 0;
		} catch(Exception e) {
			Kill($"{e.GetBaseException().Message}\n\n{e}", 2);
		}
		
		Interface.UiBuilder.Draw += Draw;
		Interface.UiBuilder.OpenMainUi += OpenConf;
		
		// TODO: proper icon
		var icon_data = new byte[64 * 64 * 4];
		var icon = new TextureWrap(Textures.CreateFromRaw(new Dalamud.Interface.Textures.RawImageSpecification(64, 64, 28), icon_data, "Aetherment Icon"));
		title_entry ??= TitleMenu.AddEntry(1, "Manage Aetherment", icon, OpenConf);
		
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
			try{Native.destroy(state);}catch{}
		
		Interface.UiBuilder.Draw -= Draw;
		Interface.UiBuilder.OpenMainUi -= OpenConf;
		
		if(title_entry != null) {
			TitleMenu.RemoveEntry(title_entry);
			title_entry = null;
		}
		
		penumbra.Dispose();
		uicolor.Dispose();
		
		Commands.RemoveHandler(maincommand);
		Commands.RemoveHandler(texfindercommand);
		
		// if(watcher != null)
		// 	watcher.Dispose();
		state = 0;
		
		Native.Free();
	}
	
	private void OpenConf() {
		OnCommand(maincommand, "");
	}
	
	private void Draw() {
		if(open) {
			ImGui.SetNextWindowPos(new Vector2(50), ImGuiCond.FirstUseEver);
			ImGui.SetNextWindowSize(new Vector2(200), ImGuiCond.FirstUseEver);
			ImGui.Begin("Aetherment", ref open);
			if(state != 0) {
				try {
					var io = ImGui.GetIO();
					var pos = ImGui.GetCursorScreenPos();
					var size = ImGui.GetContentRegionAvail();
					var focused = ImGui.IsWindowFocused();
					var drawlist = ImGui.GetWindowDrawList();
					
					ImGui.InvisibleButton("input_blocker", size);
					if(ImGui.IsItemHovered()) {
						io.ConfigFlags = io.ConfigFlags | ImGuiConfigFlags.NoMouseCursorChange;
						reset_cursor = true;
					} else if(reset_cursor) {
						io.ConfigFlags = io.ConfigFlags & ~ImGuiConfigFlags.NoMouseCursorChange;
						reset_cursor = false;
					}
					
					nint input_buf_ptr = 0;
					nint input_buf_len = io.InputQueueCharacters.Size;
					if(input_buf_len > 0) {
						input_buf_ptr = Marshal.AllocHGlobal(input_buf_len  * 2);
						
						for(var i = 0; i < io.InputQueueCharacters.Size; i++)
							unsafe{*(ushort*)(input_buf_ptr + i * 2) = io.InputQueueCharacters[i];}
					}
					
					byte set_keyboard_focus = 0;
					nint tex;
					unsafe {
						tex = Native.draw(state, device, new() {
							width = size.X,
							height = size.Y,
							mouse_x = io.MousePos.X - pos.X,
							mouse_y = io.MousePos.Y - pos.Y,
							wheel_x = io.MouseWheelH,
							wheel_y = io.MouseWheel,
							mods = (uint)(
								(io.KeyAlt ? 0b001 : 0) +
								(io.KeyCtrl ? 0b010 : 0) +
								(io.KeyShift ? 0b100 : 0) +
								
								(io.MouseDown[0] ? 0b00001000 : 0) +
								(io.MouseDown[1] ? 0b00010000 : 0) +
								(io.MouseDown[2] ? 0b00100000 : 0) +
								(io.MouseDown[3] ? 0b01000000 : 0) +
								(io.MouseDown[4] ? 0b10000000 : 0) +
								
								(focused ? 0b1_00000000 : 0)),
							input_buf_ptr = input_buf_ptr,
							input_buf_len = input_buf_len,
							
							ui_scale = Dalamud.Interface.Utility.ImGuiHelpers.GlobalScale,
							
							set_keyboard_focus = &set_keyboard_focus,
						});
					}
					
					if(set_keyboard_focus != 0)
						io.WantTextInput = true;
					
					drawlist.AddImage(new ImTextureID(tex), pos, pos + size);
					
					if(input_buf_ptr != 0)
						Marshal.FreeHGlobal(input_buf_ptr);
				} catch(Exception e) {
					Kill($"Fatal error in draw\n\n{e}", 1);
				}
			} else {
				ImGui.Text("Aetherment has encountered an unrecoverable error");
				ImGui.Text(error ?? "No Error");
			}
			ImGui.End();
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
		
		try {
			if(Native.command(state, args) == 0)
				open = !open;
		} catch(Exception e) {
			Kill($"Fatal error in command\n\n{e}", 1);
		}
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
	
	private SetNotificationDelegate setNotification;
	private unsafe delegate void SetNotificationDelegate(float progress, byte state, FFI.Str msg);
	private unsafe void SetNotification(float progress, byte state, FFI.Str msg) {
		notification ??= NotifMan.AddNotification(new Notification());
		notification.MinimizedText = msg;
		notification.Title = msg;
		notification.Progress = progress;
		if(state == 1)
			notification.Type = NotificationType.Success;
		else if(state == 2)
			notification.Type = NotificationType.Error;
		else
			notification.Type = NotificationType.None;
	}
}