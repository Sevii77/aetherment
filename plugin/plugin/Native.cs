using System.IO;
using System.Runtime.InteropServices;
using static Aetherment.Aetherment;

namespace Aetherment;

/*
	Some anti virus don't like the fact that core uses retour/region
	(ACT suffers from the same issue https://github.com/ravahn/FFXIV_ACT_Plugin/issues/324)
	this exists so that we can bypass dalamuds shadowcopy and allow users to more easily add exceptions
	for their anti virus, version updates will still break it if it doesn't support wildcards but oh well,
	it's better than it breaking every launch i gues?
*/
public static class Native {
	private static nint handle;
	
	static Native() {
		handle = LoadLibrary(Path.Join(Interface.AssemblyLocation.DirectoryName, "aetherment_core.dll"));
		
		initialize = Marshal.GetDelegateForFunctionPointer<delegate__initialize>(NativeLibrary.GetExport(handle, "initialize"));
		destroy = Marshal.GetDelegateForFunctionPointer<delegate__destroy>(NativeLibrary.GetExport(handle, "destroy"));
		command = Marshal.GetDelegateForFunctionPointer<delegate__command>(NativeLibrary.GetExport(handle, "command"));
		draw = Marshal.GetDelegateForFunctionPointer<delegate__draw>(NativeLibrary.GetExport(handle, "draw"));
		backend_penumbraipc_modchanged = Marshal.GetDelegateForFunctionPointer<delegate__backend_penumbraipc_modchanged>(NativeLibrary.GetExport(handle, "backend_penumbraipc_modchanged"));
	}
	
	public static void Free() {
		FreeLibrary(handle);
	}
	
	[DllImport("Kernel32.dll")] private static extern nint LoadLibrary(string path);
	[DllImport("Kernel32.dll")] private static extern byte FreeLibrary(nint module);
	
	public static delegate__initialize initialize;
	public delegate nint delegate__initialize(Initializers data);
	
	public static delegate__destroy destroy;
	public delegate void delegate__destroy(nint state);
	
	public static delegate__command command;
	public delegate void delegate__command(nint state, FFI.Str args);
	
	public static delegate__draw draw;
	public delegate void delegate__draw(nint state);
	
	public static delegate__backend_penumbraipc_modchanged backend_penumbraipc_modchanged;
	public delegate void delegate__backend_penumbraipc_modchanged(byte type, FFI.Str collection_id, FFI.Str mod_id);
	
	// [DllImport("aetherment_core.dll")] private static extern unsafe nint initialize(Initializers data);
	// [DllImport("aetherment_core.dll")] private static extern unsafe void destroy(nint state);
	// [DllImport("aetherment_core.dll")] private static extern unsafe void command(nint state, FFI.Str args);
	// [DllImport("aetherment_core.dll")] private static extern unsafe void draw(nint state);
	// [DllImport("aetherment_core.dll")] private static extern unsafe void backend_penumbraipc_modchanged(byte type, FFI.Str collection_id, FFI.Str mod_id);
}