using System;
using System.Text;
using System.Runtime.InteropServices;

namespace Aetherment.FFI;

// TODO: just alloc some for every string and make the rust side call back to drop it
[StructLayout(LayoutKind.Explicit)]
public struct Str {
	[FieldOffset(0x0)] private IntPtr ptr;
	[FieldOffset(0x8)] private ulong length;
	
	private const int bufLen = 65536;
	private static IntPtr buf;
	private static int bufIndex;
	
	static Str() {
		buf = Marshal.AllocHGlobal(bufLen);
		bufIndex = 0;
	}
	
	public static void Drop() {
		Marshal.FreeHGlobal(buf);
	}
	
	public Str(string str) {
		var length = Encoding.UTF8.GetByteCount(str);
		if(length > bufLen) {
			Aetherment.Logger.Error("String was longer than buffer, sending empty ffi string");
			this.ptr = buf;
			this.length = 0;
			return;
		}
		
		if(bufIndex + length > bufLen)
			bufIndex = 0;
		
		this.ptr = buf + bufIndex;
		this.length = (ulong)length;
		bufIndex += length;
		
		unsafe {
			var p = (byte*)ptr;
			fixed(char* chars = str) {
				Encoding.UTF8.GetBytes(chars, str.Length, p, length);
			}
		}
	}
	
	public static implicit operator Str(string str) => new Str(str);
	
	public static unsafe implicit operator string(Str str) {
		return Encoding.UTF8.GetString((byte*)str.ptr, (int)str.length);
	}
	
	public override string ToString() => (string)this;
	
	// public static unsafe string StrToString(byte* ptr, ulong len) {
	// 	var str = Encoding.UTF8.GetString(ptr, (int)len);
	// 	destroy_string(ptr);
	// 	return str;
	// }
	
	// [DllImport("aetherment_core.dll")] private static extern unsafe void destroy_string(byte* ptr);
}