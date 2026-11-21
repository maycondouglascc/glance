export const site = {
  url: "https://glance.maycondouglas.work/",
  github: "https://github.com/maycondouglascc/glance",
  releases: "https://github.com/maycondouglascc/glance/releases/latest",
  installer:
    "curl -fsSL https://raw.githubusercontent.com/maycondouglascc/glance/main/install.sh | sh",
};
export const copy = {
  en: {
    skip: "Skip to content",
    explore: "Features",
    install: "Install Glance",
    guide: "Installation guide",
    label: "See what your Linux apps are using",
    headline: ["Process monitor", "for Linux"],
    description:
      "See each application's CPU and memory usage at a glance.",
    cta: "Install for Linux",
    source: "Free and open source under GPL-3.0.",
    demo: "Glance application demo",
    grouped: "By application",
    raw: "All processes",
    app: "Application",
    processes: "Processes",
    hint: "Expand an application, search by name or PID, or select the information button to inspect a process.",
    illustrative: "Interactive demo with sample data",
    idle: "CPU use while minimized",
    measurement:
      "Project README figures for tray mode, with scanning paused. Memory usage is approximate.",
    memory: "Memory use while minimized",
    native: "Built for the Linux desktop",
    why: "See what each\napp is using",
    whyText:
      "Glance brings an app and its processes together, making it easier to see how much they use.",
    features: [
      [
        "See the whole app",
        "See how much CPU and memory each app uses, even when it runs several processes.",
      ],
      [
        "Find more details",
        "Open an app to see its processes, or search for one by name or PID.",
      ],
      [
        "Stop a process",
        "Stop a process when you need to. Glance also protects critical system processes.",
      ],
    ],
    installTitle: "Install Glance\non Linux",
    installText:
      "Choose your Linux system, then copy and run the command in your terminal.",
    methods: ["Universal", "Debian / Ubuntu", "Arch Linux", "From source"],
    descriptions: [
      "Installs Glance and glance-tree in ~/.local/bin without administrator access.",
      "Download the native .deb package from GitHub Releases, then install the downloaded file.",
      "Build the included PKGBUILD with makepkg.",
      "Requires Rust and the GTK4 / libadwaita development libraries.",
    ],
    copy: "Copy command",
    copied: "Command copied",
    failed: "Could not copy. Select the command to copy it manually.",
    launcher: "Press Super and search for Glance to open the application.",
    terminal: "Command-line process tree",
    terminalText:
      "Run glance-tree -c to list applications with their child processes.",
    footer: "Glance is a free, open-source app for Linux.",
    author: "Designed & built by",
    license: "GPL-3.0 license",
    notFound: "Page not found",
    notFoundText:
      "We couldn't find this page. Go back to learn about Glance and how to install it.",
    home: "Open Glance home page",
    guideTitle: "Install and\nlaunch Glance",
    guideText:
      "Choose your Linux system, install Glance, then open it from your applications menu.",
    theme: "Switch color theme",
  },
  pt: {
    skip: "Pular para o conteúdo",
    explore: "Recursos",
    install: "Instalar Glance",
    guide: "Guia de instalação",
    label: "Veja o que seus aplicativos usam no Linux",
    headline: ["Monitor de", "processos Linux"],
    description:
      "Veja o uso de CPU e memória de cada aplicativo de forma simples e rápida",
    cta: "Instalar no Linux",
    source: "Gratuito e de código aberto sob a GPL-3.0.",
    demo: "Demonstração do Glance",
    grouped: "Por aplicativo",
    raw: "Todos os processos",
    app: "Aplicativo",
    processes: "Processos",
    hint: "Expanda um aplicativo, busque por nome ou PID ou selecione o botão de informações para inspecionar um processo.",
    illustrative: "Demonstração interativa com dados de exemplo",
    idle: "Uso de CPU minimizado",
    measurement:
      "Valores do README para o modo bandeja, com varredura pausada. O uso de memória é aproximado.",
    memory: "Uso de memória minimizado",
    native: "Feito para o desktop Linux",
    why: "Veja o que cada\naplicativo usa",
    whyText:
      "O Glance reúne cada aplicativo e seus processos para você ver quanto eles usam.",
    features: [
      [
        "Veja o aplicativo inteiro",
        "Confira o uso de CPU e memória de cada aplicativo, mesmo quando ele executa vários processos.",
      ],
      [
        "Encontre mais detalhes",
        "Abra um aplicativo para ver seus processos ou busque um deles pelo nome ou PID.",
      ],
      [
        "Encerre um processo",
        "Encerre um processo quando precisar. O Glance também protege processos essenciais do sistema.",
      ],
    ],
    installTitle: "Instale o Glance\nno Linux",
    installText:
      "Escolha seu sistema Linux, copie o comando e execute-o no terminal.",
    methods: ["Universal", "Debian / Ubuntu", "Arch Linux", "Código-fonte"],
    descriptions: [
      "Instala o Glance e o glance-tree em ~/.local/bin sem acesso de administrador.",
      "Baixe o pacote .deb nativo no GitHub Releases e instale o arquivo baixado.",
      "Compile o PKGBUILD incluído com makepkg.",
      "Requer Rust e as bibliotecas de desenvolvimento GTK4 e libadwaita.",
    ],
    copy: "Copiar comando",
    copied: "Comando copiado",
    failed:
      "Não foi possível copiar. Selecione o comando para copiá-lo manualmente.",
    launcher: "Pressione Super e busque por Glance para abrir o aplicativo.",
    terminal: "Árvore de processos no terminal",
    terminalText:
      "Execute glance-tree -c para listar aplicativos e seus processos filhos.",
    footer: "Glance é um aplicativo gratuito e de código aberto para Linux.",
    author: "Design e desenvolvimento por",
    license: "Licença GPL-3.0",
    notFound: "Página não encontrada",
    notFoundText:
      "Não encontramos esta página. Volte ao início para conhecer o Glance e saber como instalá-lo.",
    home: "Abrir página inicial do Glance",
    guideTitle: "Instale e\nabra o Glance",
    guideText:
      "Escolha seu sistema Linux, instale o Glance e abra pelo menu de aplicativos.",
    theme: "Alternar tema de cores",
  },
};
export const methods = [
  site.installer,
  "sudo apt install ./glance_*_amd64.deb",
  "git clone https://github.com/maycondouglascc/glance.git\ncd glance/packaging/aur\nmakepkg -si",
  "cargo install --git https://github.com/maycondouglascc/glance.git glance",
];
