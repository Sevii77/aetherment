using System.Text;
using System.Runtime.InteropServices;
using System.Diagnostics;
using System.Collections.Generic;

namespace Aetherment.FFI;

[StructLayout(LayoutKind.Explicit)]
public struct Str {
	[FieldOffset(0x0)] private nint ptr;
	[FieldOffset(0x8)] private ulong length;
	
	// if we dropped things every frame it might cause issues due to multi-threading bs, thats why time
	private static Queue<((nint, ulong), Stopwatch)> strings;
	
	static Str() {
		strings = new();
	}
	
	public static void HandleResources() {
		while(strings.TryPeek(out var s)) {
			if(s.Item2.ElapsedMilliseconds < 100)
				break;
			
			var str = strings.Dequeue();
			Marshal.FreeHGlobal(str.Item1.Item1);
			// Aetherment.Logger.Debug($"destroyed string of {str.Item1.Item2} bytes");
		}
	}
	
	public static void Drop() {
		while(strings.TryDequeue(out var str))
			Marshal.FreeHGlobal(str.Item1.Item1);
	}
	
	public Str(string str) {
		var len = Encoding.UTF8.GetByteCount(str);
		
		ptr = Marshal.AllocHGlobal(len);;
		length = (ulong)len;
		
		unsafe {
			var p = (byte*)ptr;
			fixed(char* chars = str) {
				Encoding.UTF8.GetBytes(chars, str.Length, p, len);
			}
		}
		
		strings.Enqueue(((ptr, length), Stopwatch.StartNew()));
		// Aetherment.Logger.Debug($"created string of {len} bytes");
	}
	
	public static implicit operator Str(string str) => new Str(str);
	
	public static unsafe implicit operator string(Str str) {
		return Encoding.UTF8.GetString((byte*)str.ptr, (int)str.length);
	}
	
	public override string ToString() => (string)this;
}