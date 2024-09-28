namespace Aetherment;

public class Issue {
	private byte uires;
	private byte theme;
	
	public Issue() {
		// we get these now since they require a restart, we dont want the live value
		// might still break if aetherment is loaded after the user changed the setting
		// without restarting but oh well.
		uires = (byte)GetSetting("UiAssetType")!;
		theme = (byte)GetSetting("ColorThemeType")!;
		
		getUiResolution = GetUiResolution;
		getUiTheme = GetUiTheme;
	}
	
	private unsafe static uint? GetSetting(string name) {
		var c = FFXIVClientStructs.FFXIV.Client.System.Framework.Framework.Instance()->SystemConfig.SystemConfigBase.ConfigBase;
		for(var i = 0; i < c.ConfigCount; ++i) {
			var entry = c.ConfigEntry[i];
			if((nint)entry.Name == 0)
				continue;
			
			var s = System.Text.Encoding.UTF8.GetString(entry.Name, 128);
			s = s.Substring(0, s.IndexOf('\0'));
			if(s != name)
				continue;
			
			return entry.Value.UInt;
		}
		
		return null;
	}
	
	public GetUiResolutionDelegate getUiResolution;
	public delegate byte GetUiResolutionDelegate();
	public byte GetUiResolution() {
		return uires;
	}
	
	public GetUiThemeDelegate getUiTheme;
	public delegate byte GetUiThemeDelegate();
	public byte GetUiTheme() {
		return theme;
	}
}