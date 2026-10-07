export const pt = {
  settings: {
    label: 'Configurações',
  },
  language: {
    selectorLabel: 'Selecionar idioma',
    english: 'EN',
    portuguese: 'PT',
    englishLong: 'English',
    portugueseLong: 'Português',
  },
  theme: {
    toggle: 'Alternar tema',
    light: 'Claro',
    dark: 'Escuro',
    system: 'Sistema',
  },
  intro: {
    name: 'Glance',
    badge: 'v0.1.0',
    title: 'Glance. Monitor de processos agrupados para Linux.',
    description:
      'Um utilitário rápido e enxuto para Linux que agrupa processos em execução por aplicativo com progressive disclosure. 0% de CPU ociosa e ~65 MB de RAM no tray.',
    quickInstall: 'Instalação rápida em ~/.local/bin',
    copy: 'Copiar',
    copied: 'Copiado!',
  },
  demo: {
    badge: 'Interface Interativa',
    hint: 'Clique nas setas para expandir, busque por aplicativo ou PID, ou clique em (i) para inspecionar.',
    searchPlaceholder: 'Buscar aplicativos ou PID...',
    colApp: 'APLICATIVO',
    colCpu: 'CPU',
    colRam: 'RAM',
    secApp: 'APLICAÇÕES',
    secBg: 'SEGUNDO PLANO',
    secSys: 'SISTEMA',
    procs: '{count} procs',
    inspect: 'Inspecionar detalhes',
    terminate: 'Encerrar processo (SIGTERM)',
    reset: 'Restaurar demo',
    status: 'CPU {cpu}% · RAM {ram} GB · {procs} processos',
    detailsTitle: 'Telemetria do Processo',
    pid: 'PID',
    ppid: 'PPID',
    state: 'Estado',
    threads: 'Threads',
    cmdline: 'Linha de Comando',
    close: 'Fechar',
  },
  value: {
    title: 'Por que usar o Glance',
    subtitle:
      'Monitores tradicionais sobrecarregam sua tela com 80 linhas soltas de um mesmo navegador. O Glance traz clareza sem abrir mão do desempenho.',
    groupingTitle: 'Agrupamento por Aplicativo',
    groupingDesc:
      'Processos relacionados se organizam sob o app pai usando escopos de cgroups do systemd e entradas desktop.',
    efficiencyTitle: '0,00% de CPU Ociosa',
    efficiencyDesc:
      'Zera o uso de CPU quando minimizado ou fechado para a bandeja com esperas de kernel bloqueantes. Usa ~65 MB de RSS com mimalloc e aceleração Cairo 2D.',
    safetyTitle: 'Sinais Seguros de Processo',
    safetyDesc:
      'Utiliza Linux pidfd e validação de horário de início para evitar reaproveitamento acidental de PID. Nunca fecha o processo errado.',
    disclosureTitle: 'Progressive Disclosure',
    disclosureDesc:
      'Resumo limpo no topo por padrão. Dê duplo clique ou expanda apenas quando precisar ver renderers, extensões ou processos auxiliares.',
  },
  install: {
    title: 'Começar a Usar',
    subtitle: 'Disponível para todas as distribuições Linux modernas. Escolha o método de sua preferência:',
    tabCurl: 'Script Universal',
    tabDeb: 'Ubuntu / Debian',
    tabAur: 'Arch Linux',
    tabCargo: 'Cargo',
    cliHintTitle: 'Terminal Companion:',
    cliHintText:
      'O Glance inclui o glance-tree para quem prefere a linha de comando. Execute glance-tree -c para visualizar a árvore expandida no terminal.',
  },
  author: {
    title: 'Sobre o criador',
    bio:
      'Criado e desenvolvido por Maycon Douglas, Product Designer. O Glance nasceu para resolver o incômodo diário com listas caóticas de processos no Linux, unindo design de interface limpo, engenharia de software e performance.',
    portfolioLink: 'Conheça meu portfólio de design →',
  },
  footer: {
    contact: 'Contato & Links',
    portfolio: 'Portfólio',
    github: 'GitHub',
    linkedin: 'LinkedIn',
    email: 'Copiar Email',
    license: 'Licença GPL-3.0',
  },
}
