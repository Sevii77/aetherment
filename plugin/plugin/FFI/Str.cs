using System.Text;
using System.Runtime.InteropServices;

namespace Aetherment.FFI;

[StructLayout(LayoutKind.Explicit)]
public struct Str {
	[FieldOffset(0x0)] private nint ptr;
	[FieldOffset(0x8)] private nint length;
	
	static Str() {
		drop = Drop;
	}
	
	public Str(string str) {
		length = Encoding.UTF8.GetByteCount(str);
		ptr = Marshal.AllocHGlobal(length);;
		Aetherment.Logger.Verbose($"creating ffi.str {ptr:X} [{length}]");
		
		unsafe {
			var p = (byte*)ptr;
			fixed(char* chars = str) {
				Encoding.UTF8.GetBytes(chars, str.Length, p, (int)length);
			}
		}
	}
	
	public static DropDelegate drop;
	public delegate void DropDelegate(nint ptr, nint length);
	public static void Drop(nint ptr, nint length) {
		Aetherment.Logger.Verbose($"dropping ffi.str {ptr:X} [{length}]");
		Marshal.FreeHGlobal(ptr);
	}
	
	public static implicit operator Str(string str) => new Str(str);
	
	public static unsafe implicit operator string(Str str) {
		Aetherment.Logger.Verbose($"casting ffi.str {str.ptr:X} [{str.length}]");
		return Encoding.UTF8.GetString((byte*)str.ptr, (int)str.length);
	}
	
	public override string ToString() => (string)this;
}