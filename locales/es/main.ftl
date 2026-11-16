## Mari Encoder — español

## Pestañas
tab-queue = Cola
tab-presets = Presets
tab-settings = Ajustes
tab-about = Acerca de

## Avisos generales
toast-preset-missing = Ese preset ya no existe: elige otro de la lista.
toast-create-preset-first = Crea un preset antes de añadir archivos.
toast-save-failed = No se pudo guardar la configuración: { $error }
toast-menu-update-failed = No se pudo actualizar el menú contextual: { $error }
toast-modern-update-failed = No se pudo actualizar el menú de Windows 11: { $error }
toast-menu-install-failed = No se pudo instalar el menú: { $error }
toast-menu-modern-failed = Menú clásico instalado. Falló el menú de Windows 11: { $error }
toast-menu-installed = Menú contextual instalado.
toast-menu-removed = Menú contextual eliminado.
toast-menu-remove-failed = No se pudo eliminar el menú: { $error }
ffmpeg-missing-banner = No se encontró ffmpeg: revisa Ajustes

## Cola
dialog-add-videos = Añadir vídeos
filter-videos = Vídeos
filter-all-files = Todos los archivos
queue-add-files = ➕  Añadir archivos
queue-preset = Preset
queue-apply-all = Aplicar a todos
queue-apply-all-tip = Usa este preset en todos los archivos que aún no han empezado
job-remove = Quitar de la lista
job-cancel = Cancelar
job-open-folder = Abrir carpeta
job-open-folder-tip = Muestra el archivo generado en el Explorador
job-retry = Reintentar
queue-count = { $count ->
    [0] Ningún archivo
    [one] 1 archivo
   *[other] { $count } archivos
}
queue-active = { $count } en curso
queue-done = { $count ->
    [one] 1 terminado
   *[other] { $count } terminados
}
queue-failed = { $count } con error
queue-encode-one = ▶  Codificar
queue-encode-many = ▶  Codificar { $count } archivos
queue-cancel-all = Cancelar todo
queue-clear-finished = Limpiar terminados
queue-empty-title = Arrastra vídeos o carpetas aquí
queue-empty-hint = o haz clic derecho en un vídeo en el Explorador → { $menu }
compact-done = Terminado ✔
compact-open-full = Abrir ventana completa
drop-overlay = Suelta para añadir a la cola

## Estado de cada archivo
status-ready = { $size } · listo para codificar
status-queued = { $preset } · en cola
status-progress = { $preset } · { $percent }%
status-speed = { $speed }x
status-eta = faltan { $time }
status-encoding = { $preset } · codificando…
status-done = { $preset } · { $from } → { $to }
status-error = Error: { $message }
status-unknown-error = error desconocido
status-cancelled = { $preset } · cancelado

## Presets
presets-empty = No hay presets. Crea uno con “Nuevo”.
presets-new = ➕ Nuevo
presets-duplicate = Duplicar
presets-copy-name = { $name } (copia)
presets-delete = Eliminar
presets-move-up = Subir en el menú
presets-move-down = Bajar en el menú
presets-not-in-menu = fuera del menú
preset-new-name = Nuevo preset

editor-general = General
editor-name = Nombre
editor-format = Formato
editor-context-menu = Menú contextual
editor-show-in-menu = Mostrar en el clic derecho
editor-gif = GIF
editor-max-size = Tamaño máximo
editor-max-fps = FPS máximo
editor-video = Vídeo
editor-codec = Códec
editor-encoder = Codificador
editor-rate-control = Control de tasa
editor-quality = Calidad
editor-quality-hint = menor = mejor · H.264 18–28 · HEVC 22–30 · AV1/VP9 28–40
editor-video-bitrate = Bitrate de vídeo
editor-file-size = Tamaño del archivo
editor-file-size-hint = aproximado (una pasada)
editor-speed = Velocidad
editor-max-resolution = Resolución máxima
editor-audio = Audio
editor-bitrate = Bitrate
editor-output = Salida
editor-suffix = Sufijo del nombre
editor-folder = Carpeta
editor-folder-hint = La misma carpeta del original
editor-output-example = video{ $suffix }.{ $ext }  ·  también vale el nombre de una subcarpeta
editor-extra-args = Argumentos extra de ffmpeg
editor-optional = opcional
editor-command = Comando
browse = Examinar…
original = Original

## Opciones de los presets
container-gif = GIF animado
container-mp3 = MP3 (solo audio)
container-m4a = M4A (solo audio)
container-wav = WAV (solo audio)
codec-copy = Copiar (sin recodificar)
hw-windows = Windows (automático)
hw-nvidia = GPU NVIDIA (NVENC)
hw-intel = GPU Intel (Quick Sync)
hw-amd = GPU AMD (AMF)
rate-quality = Calidad constante (CRF)
rate-bitrate = Bitrate fijo
rate-target-size = Tamaño de archivo objetivo
speed-fast = Rápido
speed-medium = Equilibrado
speed-slow = Lento (mejor compresión)
audio-pcm = PCM (sin compresión)
audio-none = Sin audio

## Ajustes
settings-menu = Menú contextual de Windows
settings-menu-store = ● Activo: instalado con la app desde Microsoft Store
settings-menu-store-hint = Clic derecho en un vídeo → { $menu }, con un elemento por preset. En Windows 11 aparece en el menú principal; Windows decide su posición.
settings-menu-both = ● Instalado: menú de Windows 11 y menú clásico
settings-menu-modern-only = ● Instalado: solo en el menú de Windows 11
settings-menu-classic-only = ● Instalado: solo en el menú clásico
settings-menu-not-installed = ● No instalado
settings-install = Instalar
settings-update = Actualizar
settings-remove = Quitar
settings-menu-hint = Clic derecho en un vídeo → { $menu }, con un elemento por preset. En el menú clásico (“Mostrar más opciones” o Mayús + clic derecho) queda arriba del todo; en el menú principal de Windows 11 el sistema decide la posición. No requiere administrador.
settings-modern-needs-devmode = El menú principal de Windows 11 no está registrado: sin firma, requiere el Modo de desarrollador. Haz clic en Actualizar para reintentar.
settings-modern-unavailable = Menú principal de Windows 11 no disponible: requiere Windows 11 y MariEncoderMenu.dll en la carpeta de la app.
settings-extensions-store = Extensiones de vídeo
settings-extensions = Extensiones con el menú
settings-extensions-store-hint = Separadas por comas. Definen qué se añade al arrastrar una carpeta a la ventana.
settings-extensions-hint = Separadas por comas. También definen qué se añade al arrastrar una carpeta.
settings-ffmpeg = ffmpeg
settings-ffmpeg-found = ● Encontrado: { $path }
settings-ffmpeg-missing = ● No encontrado
settings-ffmpeg-folder = Carpeta de ffmpeg
settings-ffmpeg-auto = Automático (junto a la app o en el PATH)
settings-behavior = Comportamiento
settings-close-when-done = Cerrar la ventana rápida al terminar sin errores
settings-default-preset = Preset predeterminado de la ventana
settings-language = Idioma
settings-config-path = Configuración: { $path }
lang-system = Idioma de Windows

## Acerca de
about-version = Versión { $version }
about-desc = Codifica vídeos con un clic, directamente desde el clic derecho de Windows.
about-dedication = Dedicado a Mariana Lima, con mucho amor.
about-author-title = CREADO POR
about-author-bio = Director de arte, motion designer y creador de productos digitales, con más de 15 años entre diseño, tecnología y comunicación. Fundador de EvoMind, un espacio de trabajo visual con IA.
about-link-github = Código en GitHub
about-support-title = APOYA EL PROYECTO
about-support-desc = { $app } es gratuito, sin anuncios y sin versión de pago. Si te resulta útil, considera apoyarlo.
about-donate = ♥ Donar con PayPal
about-updates-title = ACTUALIZACIONES
about-updates-store = Las actualizaciones llegan automáticamente a través de Microsoft Store.
about-updates-github = Ver versiones en GitHub
about-licenses-title = LICENCIAS
about-licenses-desc = Los vídeos se convierten con FFmpeg (LGPL v3), que acompaña a la app como programa independiente. H.264, HEVC y AAC usan los codificadores del propio Windows o de tu tarjeta gráfica.
about-ffmpeg-source = Código fuente del FFmpeg incluido
about-license = Licencia GPLv3  •  © 2026 Flávio Takemoto

## Errores de codificación
err-ffmpeg-start = No se pudo iniciar ffmpeg: { $error }
err-ffmpeg-failed = ffmpeg terminó con un error.
err-ffmpeg-missing = No se encontró ffmpeg. Indica su carpeta en Ajustes.
err-input-missing = No se encontró el archivo de entrada.
err-output-folder = No se pudo crear la carpeta de salida: { $error }
err-hevc-needs-gpu = HEVC mediante Windows necesita una tarjeta gráfica con codificador HEVC: usa H.264 o AV1.
err-no-encoder = No hay ningún codificador disponible para este preset.
note-gpu-fallback = GPU no disponible: usando el codificador de Windows
err-devmode = { $error } (sin firma, el registro requiere el Modo de desarrollador de Windows)
err-exe-dir = no se encontró la carpeta del ejecutable
err-powershell = no se pudo ejecutar PowerShell: { $error }
