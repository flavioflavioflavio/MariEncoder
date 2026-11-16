// Nomes e identificadores do produto, compartilhados pelo app e pela DLL do menu
// (incluído com `include!`).

/// Nome completo: Store, menu Iniciar, janelas.
pub const APP_NAME: &str = "Mari Encoder";
/// Nome curto, usado no menu de botão direito.
pub const MENU_NAME: &str = "Mari Encoder";
/// Executável do app (na mesma pasta da DLL do menu).
pub const EXE_NAME: &str = "MariEncoder.exe";
/// Pasta dentro de %APPDATA% com o config.json.
pub const DATA_DIR: &str = "MariEncoder";
/// Pasta usada pelas versões anteriores ao novo nome (migrada na primeira execução).
pub const LEGACY_DATA_DIR: &str = "QuickEncoder";

/// Classe COM do menu na versão do GitHub (pacote esparso registrado pelo app).
pub const CLSID_DEV: &str = "21811E09-8EB0-4D19-8AA7-09A4CE67FDE7";
/// Classe COM do menu no pacote da Microsoft Store (declarada no AppxManifest).
pub const CLSID_STORE: &str = "982BFDCD-9DF9-446A-8312-63DB1B108ED7";
