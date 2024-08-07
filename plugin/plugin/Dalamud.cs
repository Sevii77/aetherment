using System;
using System.Collections.Generic;
using System.Reflection;
using Dalamud.Interface.Style;
using Newtonsoft.Json;

namespace Aetherment;

public class DalamudStyle {
	public DalamudStyle() {
		addStyle = AddStyle;
	}
	
	public AddStyleDelegate addStyle;
	public delegate void AddStyleDelegate(FFI.Str json);
	public static void AddStyle(FFI.Str json) {
		try {
			var style = StyleModelV1.DalamudStandard;
			JsonConvert.PopulateObject(json, style);
			
			style.Apply();
			
			// reflection garbage to save it
			var ass = typeof(Dalamud.Configuration.PluginConfigurations).Assembly;
			var t = ass.GetType("Dalamud.Configuration.Internal.DalamudConfiguration")!;
			var dalamudConfig = ass.GetType("Dalamud.Service`1")!
				.MakeGenericType(t)!
				.GetMethod("Get")!
				.Invoke(null, BindingFlags.Default, null, new object[0], null);
			
			PropertyInfo savedStylesInfo = t.GetProperty("SavedStyles")!;
			List<StyleModel> savedStyles = (List<StyleModel>)savedStylesInfo.GetValue(dalamudConfig)!;
			for(int i = 0; i < savedStyles.Count; i++)
				if(savedStyles[i].Name == style.Name) {
					savedStyles[i] = style;
					goto Save;
				}
			
			savedStyles.Add(style);
			
			Save:
			savedStylesInfo.SetValue(dalamudConfig, savedStyles);
			
			PropertyInfo chosenStyleInfo = t.GetProperty("ChosenStyle")!;
			chosenStyleInfo.SetValue(dalamudConfig, style.Name);
			
			t.GetMethod("QueueSave")!.Invoke(dalamudConfig, new object[0]);
		} catch(Exception e) {
			Aetherment.Logger.Error($"Failed adding style , {e}");
		}
	}
}