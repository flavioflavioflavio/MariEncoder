## Mari Encoder — português (Brasil)

## Abas
tab-queue = Fila
tab-presets = Presets
tab-settings = Ajustes
tab-about = Sobre

## Avisos gerais
toast-preset-missing = Esse preset não existe mais — escolha outro na lista.
toast-create-preset-first = Crie um preset antes de adicionar arquivos.
toast-save-failed = Não foi possível salvar a configuração: { $error }
toast-menu-update-failed = Não foi possível atualizar o menu contextual: { $error }
toast-modern-update-failed = Não foi possível atualizar o menu do Windows 11: { $error }
toast-menu-install-failed = Falha ao instalar o menu: { $error }
toast-menu-modern-failed = Menu clássico instalado. Menu do Windows 11 falhou: { $error }
toast-menu-installed = Menu contextual instalado.
toast-menu-removed = Menu contextual removido.
toast-menu-remove-failed = Falha ao remover o menu: { $error }
ffmpeg-missing-banner = ffmpeg não encontrado — veja Ajustes

## Fila
dialog-add-videos = Adicionar vídeos
filter-videos = Vídeos
filter-all-files = Todos os arquivos
queue-add-files = ➕  Adicionar arquivos
queue-preset = Preset
queue-apply-all = Aplicar a todos
queue-apply-all-tip = Usa este preset em todos os arquivos que ainda não começaram
job-remove = Remover da lista
job-cancel = Cancelar
job-open-folder = Abrir pasta
job-open-folder-tip = Mostra o arquivo gerado no Explorer
job-retry = Tentar de novo
queue-count = { $count ->
    [0] Nenhum arquivo
    [one] 1 arquivo
   *[other] { $count } arquivos
}
queue-active = { $count } em andamento
queue-done = { $count ->
    [one] 1 concluído
   *[other] { $count } concluídos
}
queue-failed = { $count } com erro
queue-encode-one = ▶  Encodar
queue-encode-many = ▶  Encodar { $count } arquivos
queue-cancel-all = Cancelar tudo
queue-clear-finished = Limpar concluídos
queue-empty-title = Arraste vídeos ou pastas para cá
queue-empty-hint = ou use o botão direito num vídeo no Explorer → { $menu }
compact-done = Concluído ✔
compact-open-full = Abrir janela completa
drop-overlay = Solte para adicionar à fila

## Status de cada arquivo
status-ready = { $size } · pronto para encodar
status-queued = { $preset } · na fila
status-progress = { $preset } · { $percent }%
status-speed = { $speed }x
status-eta = faltam { $time }
status-encoding = { $preset } · encodando…
status-done = { $preset } · { $from } → { $to }
status-error = Erro: { $message }
status-unknown-error = erro desconhecido
status-cancelled = { $preset } · cancelado

## Presets
presets-empty = Nenhum preset. Crie um em “Novo”.
presets-new = ➕ Novo
presets-duplicate = Duplicar
presets-copy-name = { $name } (cópia)
presets-delete = Excluir
presets-move-up = Subir no menu
presets-move-down = Descer no menu
presets-not-in-menu = fora do menu
preset-new-name = Novo preset

editor-general = Geral
editor-name = Nome
editor-format = Formato
editor-context-menu = Menu contextual
editor-show-in-menu = Mostrar no botão direito
editor-gif = GIF
editor-max-size = Tamanho máximo
editor-max-fps = FPS máximo
editor-video = Vídeo
editor-codec = Codec
editor-encoder = Encoder
editor-rate-control = Controle de taxa
editor-quality = Qualidade
editor-quality-hint = menor = melhor · H.264 18–28 · HEVC 22–30 · AV1/VP9 28–40
editor-video-bitrate = Bitrate de vídeo
editor-file-size = Tamanho do arquivo
editor-file-size-hint = aproximado (uma passada)
editor-speed = Velocidade
editor-max-resolution = Resolução máxima
editor-audio = Áudio
editor-bitrate = Bitrate
editor-output = Saída
editor-suffix = Sufixo do nome
editor-folder = Pasta
editor-folder-hint = Mesma pasta do original
editor-output-example = vídeo{ $suffix }.{ $ext }  ·  um nome de subpasta também funciona
editor-extra-args = Args extras do ffmpeg
editor-optional = opcional
editor-command = Comando
browse = Procurar…
original = Original

## Opções dos presets
container-gif = GIF animado
container-mp3 = MP3 (só áudio)
container-m4a = M4A (só áudio)
container-wav = WAV (só áudio)
codec-copy = Copiar (sem reencode)
hw-windows = Windows (automático)
hw-nvidia = GPU NVIDIA (NVENC)
hw-intel = GPU Intel (Quick Sync)
hw-amd = GPU AMD (AMF)
rate-quality = Qualidade constante (CRF)
rate-bitrate = Bitrate fixo
rate-target-size = Tamanho-alvo do arquivo
speed-fast = Rápido
speed-medium = Equilibrado
speed-slow = Lento (melhor compressão)
audio-pcm = PCM (sem compressão)
audio-none = Sem áudio

## Ajustes
settings-menu = Menu contextual do Windows
settings-menu-store = ● Ativo — instalado junto com o app pela Microsoft Store
settings-menu-store-hint = Botão direito num vídeo → { $menu }, com um item por preset. No Windows 11 ele aparece no menu principal; a posição é decidida pelo Windows.
settings-menu-both = ● Instalado — menu do Windows 11 e menu clássico
settings-menu-modern-only = ● Instalado — só no menu do Windows 11
settings-menu-classic-only = ● Instalado — só no menu clássico
settings-menu-not-installed = ● Não instalado
settings-install = Instalar
settings-update = Atualizar
settings-remove = Remover
settings-menu-hint = Botão direito num vídeo → { $menu }, com um item por preset. No menu clássico (“Mostrar mais opções” ou Shift + botão direito) ele fica no topo; no menu principal do Windows 11 é o sistema que decide a posição. Não precisa de administrador.
settings-modern-needs-devmode = O menu principal do Windows 11 não está registrado: sem assinatura, ele exige o Modo de Desenvolvedor ligado. Clique em Atualizar para tentar de novo.
settings-modern-unavailable = Menu principal do Windows 11 indisponível: precisa do Windows 11 e da MariEncoderMenu.dll na pasta do app.
settings-extensions-store = Extensões de vídeo
settings-extensions = Extensões com o menu
settings-extensions-store-hint = Separadas por vírgula. Definem o que entra ao arrastar uma pasta para a janela.
settings-extensions-hint = Separadas por vírgula. Também definem o que entra ao arrastar uma pasta.
settings-ffmpeg = ffmpeg
settings-ffmpeg-found = ● Encontrado: { $path }
settings-ffmpeg-missing = ● Não encontrado
settings-ffmpeg-folder = Pasta do ffmpeg
settings-ffmpeg-auto = Automático (ao lado do app ou no PATH)
settings-behavior = Comportamento
settings-close-when-done = Fechar a janela rápida quando terminar sem erros
settings-default-preset = Preset padrão da janela
settings-language = Idioma
settings-config-path = Configuração: { $path }
lang-system = Idioma do Windows

## Sobre
about-version = Versão { $version }
about-desc = Encode de vídeo em um clique, direto do botão direito do Windows.
about-dedication = Dedicado a Mariana Lima, com muito amor.
about-author-title = CRIADO POR
about-author-bio = Diretor de arte, motion designer e criador de produtos digitais, com mais de 15 anos entre design, tecnologia e comunicação. Fundador da EvoMind, um espaço visual de trabalho com IA.
about-link-github = Código no GitHub
about-support-title = APOIE O PROJETO
about-support-desc = O { $app } é gratuito, sem anúncios e sem versão paga. Se ele te ajuda, considere apoiar.
about-donate = ♥ Doar com PayPal
about-updates-title = ATUALIZAÇÕES
about-updates-store = As atualizações chegam automaticamente pela Microsoft Store.
about-updates-github = Ver versões no GitHub
about-licenses-title = LICENÇAS
about-licenses-desc = Os vídeos são convertidos pelo FFmpeg (LGPL v3), que acompanha o app como programa separado. H.264, HEVC e AAC usam os encoders do próprio Windows ou da sua placa de vídeo.
about-ffmpeg-source = Código-fonte do FFmpeg incluído
about-license = Licença GPLv3  •  © 2026 Flávio Takemoto

## Erros do encode
err-ffmpeg-start = Não foi possível iniciar o ffmpeg: { $error }
err-ffmpeg-failed = O ffmpeg terminou com erro.
err-ffmpeg-missing = ffmpeg não encontrado. Indique a pasta em Ajustes.
err-input-missing = Arquivo de entrada não encontrado.
err-output-folder = Não foi possível criar a pasta de saída: { $error }
err-hevc-needs-gpu = O HEVC pelo Windows precisa de uma placa de vídeo com encoder HEVC — use H.264 ou AV1.
err-no-encoder = Nenhum encoder disponível para este preset.
note-gpu-fallback = GPU indisponível — usando o encoder do Windows
err-devmode = { $error } (sem assinatura, o registro exige o Modo de Desenvolvedor do Windows)
err-exe-dir = pasta do executável não encontrada
err-powershell = não foi possível rodar o PowerShell: { $error }
