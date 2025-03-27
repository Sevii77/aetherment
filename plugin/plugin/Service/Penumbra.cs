using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using Dalamud.Game.ClientState.Objects.Types;
using Dalamud.Plugin.Ipc;

namespace Aetherment;

// TODO: handle exceptions and send that to the rust side

// https://github.com/Ottermandias/Penumbra.Api/blob/9472b6e327109216368c3dc1720159f5295bdb13/IPenumbraApi.cs
public unsafe class Penumbra: IDisposable {
	// private ICallGateSubscriber<string, object> postSettingsDraw;
	
	public Penumbra() {
		redraw = Redraw;
		redrawSelf = RedrawSelf;
		isEnabled = IsEnabled;
		rootPath = RootPath;
		modList = ModList;
		addModEntry = AddModEntry;
		reloadMod = ReloadMod;
		setModEnabled = SetModEnabled;
		setModPriority = SetModPriority;
		setModInherit = SetModInherit;
		setModSettings = SetModSettings;
		getModSettings = GetModSettings;
		// defaultCollection = DefaultCollection;
		getCollection = GetCollection;
		getCollections = GetCollections;
		
		// postSettingsDraw = Aetherment.Interface.GetIpcSubscriber<string, object>("Penumbra.PostSettingsDraw");
		// postSettingsDraw.Subscribe(DrawSettings);
		
		modSettingChanged = Aetherment.Interface.GetIpcSubscriber<int, Guid, string, bool, object>("Penumbra.ModSettingChanged.V5");
		modSettingChanged.Subscribe(ModSettingChanged);
	}
	
	public void Dispose() {
		// postSettingsDraw.Unsubscribe(DrawSettings);
		modSettingChanged.Unsubscribe(ModSettingChanged);
	}
	
	// private static void DrawSettings(string id) {
	// 	if(Aetherment.state == IntPtr.Zero) return;
	// 	
	// 	try {
	// 		draw_settings(Aetherment.state, id);
	// 	} catch(Exception e) {
	// 		PluginLog.Error("draw_settings somehow paniced, even tho it's supposed to catch those, wtf", e);
	// 	}
	// }
	
	private ICallGateSubscriber<int, Guid, string, bool, object> modSettingChanged;
	private static void ModSettingChanged(int type, Guid collection_id, string mod_id, bool inherited) {
		// Aetherment.Logger.Debug($"{type} - {collection_id} - {mod_id} - {inherited}");
		if(!inherited && Aetherment.state != 0)
			Native.backend_penumbraipc_modchanged((byte)type, collection_id.ToString(), mod_id);
	}
	
	public RedrawDelegate redraw;
	public delegate void RedrawDelegate();
	public void Redraw() {
		try {
			Aetherment.Interface.GetIpcSubscriber<byte, object>("Penumbra.RedrawAll").InvokeAction(0);
		} catch(Exception e) {
			Aetherment.Logger.Error(e, "Penumbra IPC Redraw");
		}
	}
	
	public RedrawDelegate redrawSelf;
	public delegate void RedrawSelfDelegate();
	public void RedrawSelf() {
		try {
			Aetherment.Interface.GetIpcSubscriber<IGameObject, byte, object>("Penumbra.RedrawObject").InvokeAction(Aetherment.Objects[0]!, 0);
		} catch(Exception e) {
			Aetherment.Logger.Error(e, "Penumbra IPC RedrawSelf");
		}
	}
	
	public IsEnabledDelegate isEnabled;
	public delegate byte IsEnabledDelegate();
	public byte IsEnabled() {
		try {
			return Aetherment.Interface.GetIpcSubscriber<bool>("Penumbra.GetEnabledState").InvokeFunc() ? (byte)1 : (byte)0;
		} catch {
			return 0;
		}
	}
	
	public RootPathDelegate rootPath;
	public delegate FFI.Str RootPathDelegate();
	public FFI.Str RootPath() {
		try {
			return Aetherment.Interface.GetIpcSubscriber<string>("Penumbra.GetModDirectory").InvokeFunc();
		} catch(Exception e) {
			Aetherment.Logger.Error(e, "Penumbra IPC RootPath");
		}
		
		return "";
	}
	
	// TODO: this might return a string thats longer than the ffi.str allocated buffer if the user has an insane amount of mods. look into it
	public ModListDelegate modList;
	public delegate FFI.Str ModListDelegate();
	public FFI.Str ModList() {
		var mods_str = ""; // should use a mutable string but idc, fuck c#
		try {
			var mods = Aetherment.Interface.GetIpcSubscriber<Dictionary<string, string>>("Penumbra.GetModList").InvokeFunc();
			foreach(var (id, name) in mods) {
				if(mods_str.Length > 0)
					mods_str += "\0";
				mods_str += id;
			}
		} catch(Exception e) {
			Aetherment.Logger.Error(e, "Penumbra IPC ModList");
		}
		
		return mods_str;
	}
	
	public AddModEntryDelegate addModEntry;
	public delegate byte AddModEntryDelegate(FFI.Str id);
	public byte AddModEntry(FFI.Str id) {
		try {
			return Aetherment.Interface.GetIpcSubscriber<string, byte>("Penumbra.AddMod.V5").InvokeFunc(id);
		} catch(Exception e) {
			Aetherment.Logger.Error(e, "Penumbra IPC AddModEntry");
		}
		
		return 0;
	}
	
	// Actual reload function is broken (https://github.com/xivdev/Penumbra/issues/402) so this scuffed shit will have to do
	public ReloadModDelegate reloadMod;
	public delegate byte ReloadModDelegate(FFI.Str id);
	public byte ReloadMod(FFI.Str id) {
		try {
			return Aetherment.Interface.GetIpcSubscriber<string, string, byte>("Penumbra.ReloadMod.V5").InvokeFunc(id, "");
		} catch(Exception e) {
			Aetherment.Logger.Error(e, "Penumbra IPC ReloadMod");
		}
		
		return 0;
	}
	
	public SetModEnabledDelegate setModEnabled;
	public delegate byte SetModEnabledDelegate(FFI.Str collection, FFI.Str mod, byte enabled);
	public byte SetModEnabled(FFI.Str collection, FFI.Str mod, byte enabled) {
		try {
			return Aetherment.Interface.GetIpcSubscriber<string, string, string, bool, byte>("Penumbra.TrySetMod.V5").InvokeFunc(collection, mod, "", enabled != 0);
		} catch(Exception e) {
			Aetherment.Logger.Error(e, "Penumbra IPC SetModEnabled");
		}
		
		return 0;
	}
	
	public SetModPriorityDelegate setModPriority;
	public delegate byte SetModPriorityDelegate(FFI.Str collection, FFI.Str mod, int priority);
	public byte SetModPriority(FFI.Str collection, FFI.Str mod, int priority) {
		try {
			return Aetherment.Interface.GetIpcSubscriber<string, string, string, int, byte>("Penumbra.TrySetModPriority.V5").InvokeFunc(collection, mod, "", priority);
		} catch(Exception e) {
			Aetherment.Logger.Error(e, "Penumbra IPC SetModPriority");
		}
		
		return 0;
	}
	
	public SetModInheritDelegate setModInherit;
	public delegate byte SetModInheritDelegate(FFI.Str collection, FFI.Str mod, byte inherit);
	public byte SetModInherit(FFI.Str collection, FFI.Str mod, byte inherit) {
		try {
			return Aetherment.Interface.GetIpcSubscriber<string, string, string, int, byte>("Penumbra.TryInheritMod.V5").InvokeFunc(collection, mod, "", inherit);
		} catch(Exception e) {
			Aetherment.Logger.Error(e, "Penumbra IPC SetModInherit");
		}
		
		return 0;
	}
	
	public SetModSettingsDelegate setModSettings;
	public delegate byte SetModSettingsDelegate(FFI.Str collection, FFI.Str mod, FFI.Str option, FFI.Str sub_options_str);
	public byte SetModSettings(FFI.Str collection, FFI.Str mod, FFI.Str option, FFI.Str sub_options_str) {
		var sub_options = new List<string>();
		foreach(var sub_option in sub_options_str.ToString().Split('\0'))
			if(sub_option.Length > 0)
				sub_options.Add(sub_option);
		
		try {
			if(sub_options.Count == 1)
				return Aetherment.Interface.GetIpcSubscriber<string, string, string, string, string, byte>("Penumbra.TrySetModSetting.V5").InvokeFunc(collection, mod, "", option, sub_options[0]);
			else
				return Aetherment.Interface.GetIpcSubscriber<string, string, string, string, IReadOnlyList<string>, byte>("Penumbra.TrySetModSettings.V5").InvokeFunc(collection, mod, "", option, sub_options);
		} catch(Exception e) {
			Aetherment.Logger.Error(e, "Penumbra IPC SetModSettings");
		}
		
		return 0;
	}
	
	[StructLayout(LayoutKind.Sequential, Pack = 1)]
	public struct ModSettings {
		public byte exists;
		public byte enabled;
		public byte inherit;
		public int priority;
		public FFI.Str options;
	}
	public GetModSettingsDelegate getModSettings;
	public delegate ModSettings GetModSettingsDelegate(FFI.Str collection, FFI.Str mod, byte allow_inherit);
	public ModSettings GetModSettings(FFI.Str collection, FFI.Str mod, byte allow_inherit) {
		try {
			var (_, settings) = Aetherment.Interface.GetIpcSubscriber<string, string, string, bool, (byte, (bool, int, IDictionary<string, IList<string>>, bool)?)>("Penumbra.GetCurrentModSettings.V5").InvokeFunc(collection, mod, "", allow_inherit != 0);
			if(settings != null) {
				var (enabled, priority, options, inherit) = settings.Value;
				var options_keys = new List<string>(options.Keys);
				var options_str = "";
				for(int i = 0; i < options.Count; i++) {
					if(i > 0)
						options_str += "\0\0";
					options_str += options_keys[i];
					foreach(var sub_option in options[options_keys[i]]) {
						options_str += "\0";
						options_str += sub_option;
					}
				}
				
				return new() {
					exists = 1,
					enabled = enabled ? (byte)1 : (byte)0,
					inherit = inherit ? (byte)1 : (byte)0,
					priority = priority,
					options = options_str,
				};
			}
		} catch(Exception e) {
			Aetherment.Logger.Error(e, "Penumbra IPC GetModSettings");
		}
		
		return new() {
			exists = 0,
			enabled = 0,
			inherit = 0,
			priority = 0,
			options = "",
		};
	}
	
	public GetCollectionDelegate getCollection;
	public delegate FFI.Str GetCollectionDelegate(byte type);
	public static FFI.Str GetCollection(byte type) {
		try {
			var collection = Aetherment.Interface.GetIpcSubscriber<byte, (Guid, string)?>("Penumbra.GetCollection").InvokeFunc(type);
			if(collection.HasValue) {
				return collection.Value.Item1.ToString() + "\0" + collection.Value.Item2;
			}
		} catch(Exception e) {
			Aetherment.Logger.Error(e, "Penumbra IPC Collection");
		}
		
		return "";
	}
	
	public GetCollectionsDelegate getCollections;
	public delegate FFI.Str GetCollectionsDelegate();
	public FFI.Str GetCollections() {
		var collections_str = "";
		try {
			var collections = Aetherment.Interface.GetIpcSubscriber<Dictionary<Guid, string>>("Penumbra.GetCollections.V5").InvokeFunc();
			foreach(var (id, name) in collections) {
				if(collections_str.Length > 0)
					collections_str += "\0\0";
				collections_str += id + "\0" + name;
			}
		} catch(Exception e) {
			Aetherment.Logger.Error(e, "Penumbra IPC GetCollections");
		}
		
		return collections_str;
	}
	
	// public DefaultCollectionDelegate defaultCollection;
	// public delegate FFI.Str DefaultCollectionDelegate();
	// public FFI.Str DefaultCollection() {
	// 	return Aetherment.Interface.GetIpcSubscriber<string>("Penumbra.GetDefaultCollectionName").InvokeFunc();
	// }
	
	// public GetCollectionsDelegate getCollections;
	// public delegate FFI.Str GetCollectionsDelegate();
	// public FFI.Str GetCollections() {
	// 	var collections = Aetherment.Interface.GetIpcSubscriber<IList<string>>("Penumbra.GetCollections").InvokeFunc();
	// 	var collections_str = "";
	// 	for(int i = 0; i < collections.Count; i++) {
	// 		if(i > 0)
	// 			collections_str += "\0";
	// 		collections_str += collections[i];
	// 	}
	// 	
	// 	return collections_str;
	// }
}