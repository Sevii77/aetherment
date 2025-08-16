using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Runtime.Loader;
using static Aetherment.Aetherment;

namespace Aetherment;

// /*
// 	Some anti virus don't like the fact that core uses retour/region
// 	(ACT suffers from the same issue https://github.com/ravahn/FFXIV_ACT_Plugin/issues/324)
// 	this way we stop dalamuds shadow copy (why does it even do that, idfk)
// 	this would allow people that run into an anti virus issue to add an easy exception
// */
// public static class Native {
// 	private static nint handle;
// 	
// 	static Native() {
// 		var core_path = Path.Join(Interface.AssemblyLocation.DirectoryName, "aetherment_core.dll");
// 		if(!Path.Exists(core_path))
// 			throw new Exception("aetherment_core.dll file does not exist, this could be caused by your anti virus.");
// 		
// 		handle = LoadLibrary(core_path);
// 		if(handle == 0)
// 			throw new Exception("aetherment_core.dll failed to load", new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error()));
// 		
// 		initialize = Marshal.GetDelegateForFunctionPointer<delegate__initialize>(NativeLibrary.GetExport(handle, "initialize"));
// 		destroy = Marshal.GetDelegateForFunctionPointer<delegate__destroy>(NativeLibrary.GetExport(handle, "destroy"));
// 		command = Marshal.GetDelegateForFunctionPointer<delegate__command>(NativeLibrary.GetExport(handle, "command"));
// 		draw = Marshal.GetDelegateForFunctionPointer<delegate__draw>(NativeLibrary.GetExport(handle, "draw"));
// 		backend_penumbraipc_modchanged = Marshal.GetDelegateForFunctionPointer<delegate__backend_penumbraipc_modchanged>(NativeLibrary.GetExport(handle, "backend_penumbraipc_modchanged"));
// 	}
// 	
// 	public static void Free() {
// 		FreeLibrary(handle);
// 	}
// 	
// 	[DllImport("Kernel32.dll")] private static extern nint LoadLibrary(string path);
// 	[DllImport("Kernel32.dll")] private static extern byte FreeLibrary(nint module);
// 	
// 	public static delegate__initialize initialize;
// 	public delegate nint delegate__initialize(Initializers data);
// 	
// 	public static delegate__destroy destroy;
// 	public delegate void delegate__destroy(nint state);
// 	
// 	public static delegate__command command;
// 	public delegate void delegate__command(nint state, FFI.Str args);
// 	
// 	public static delegate__draw draw;
// 	public delegate void delegate__draw(nint state);
// 	
// 	public static delegate__backend_penumbraipc_modchanged backend_penumbraipc_modchanged;
// 	public delegate void delegate__backend_penumbraipc_modchanged(byte type, FFI.Str collection_id, FFI.Str mod_id);
// }

public static class Native {
	private static nint bcryptprimitives_handle = 0;
	
	static Native() {
		if(Dalamud.Utility.Util.IsWine()) {
			var path = Path.Join(Interface.AssemblyLocation.DirectoryName, "bcryptprimitives.dll");
			bcryptprimitives_handle = NativeLibrary.Load(path);
		}
	}
	
	public static void Free() {
		if(bcryptprimitives_handle != 0)
			FreeLibrary(bcryptprimitives_handle);
	}
	
	[DllImport("Kernel32.dll")] private static extern nint LoadLibrary(string path);
	[DllImport("Kernel32.dll")] private static extern byte FreeLibrary(nint module);
	
	[DllImport("aetherment_core.dll")] public static extern unsafe nint initialize(Initializers data);
	[DllImport("aetherment_core.dll")] public static extern unsafe void destroy(nint state);
	[DllImport("aetherment_core.dll")] public static extern unsafe byte command(nint state, FFI.Str args);
	[DllImport("aetherment_core.dll")] public static extern unsafe nint draw(nint state, nint d3d11_device, Io io);
	[DllImport("aetherment_core.dll")] public static extern unsafe void tick(nint state);
	[DllImport("aetherment_core.dll")] public static extern unsafe byte config_plugin_open_on_launch(nint state);
	[DllImport("aetherment_core.dll")] public static extern unsafe void backend_penumbraipc_modchanged(byte type, FFI.Str collection_id, FFI.Str mod_id);
}