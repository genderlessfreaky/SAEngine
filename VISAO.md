# SAEngine: visão do projeto

Qualquer pessoa cria um jogo completo, do menu inicial aos créditos, com o mundo de San Andreas. Sem escrever código.

A SAEngine é uma engine aberta, em Rust, que usa os arquivos do GTA San Andreas original de quem joga (modelos, texturas, animações, sons) como material de construção. Ela não transforma o jogo original e não distribui nada da Rockstar Games.

## Para quem é

Para iniciantes que nunca programaram. Quem cria escolhe peças prontas e monta jogos inteiros. Quem programa amplia o catálogo para todo mundo.

- **Criar sem código.** Tudo é visual: escolher, arrastar, ajustar.
- **Dois modos de editar.** Andando pelo mundo com o personagem ou num editor com o mapa visto de cima. Uma tecla troca entre os dois.
- **Peças de Lego.** Tudo no jogo é montado juntando peças pequenas e prontas: ir até o local, entrar no carro, eliminar alvo, fugir da polícia, subir de nível, abrir uma fase, mostrar um menu.
- **Comunidade amplia.** Quem não programa junta peças e salva como peça nova. Quem programa cria peças e capacidades novas em Lua.
- **Testar de qualquer ponto.** O teste pode começar do meio de uma missão ou de uma fase, sem jogar tudo de novo.

## Um jogo completo

A meta não é fazer missões soltas. É permitir criar o jogo inteiro, com tudo o que um jogo de verdade tem.

| Camada | O que a pessoa consegue fazer |
| --- | --- |
| Mundo e fases | Vários mapas e níveis ligados entre si: cidades, interiores, ilhas, arenas. |
| Missões e história | Objetivos, cenas, diálogos e escolhas, em capítulos ou em mundo aberto. |
| Eventos e regras | Coisas que acontecem sozinhas: assalto na rua, perseguição, clima, horário, território que muda de dono. |
| Progressão | Níveis, experiência, desbloqueios, conquistas, dinheiro, itens e habilidades. |
| Modos e objetivos | Campanha, corrida, sobrevivência, minigames. Regras de vitória, derrota e pontuação. |
| Interface | Menu inicial, pausa, opções, mapa, inventário, créditos e fim de jogo. |
| Salvar e continuar | Salvar o progresso em qualquer jogo criado e continuar de onde parou. |

## Capacidades que se ligam e desligam

Cada sistema do San Andreas é um módulo que quem cria liga ou desliga. O jogo só tem fome se a pessoa ligar a fome. Ao ligar uma capacidade, a engine já traz junto:

- a barra na tela;
- as peças no catálogo (lanchonete, máquina de comida, carrinho de cachorro-quente);
- as regras ajustáveis (velocidade da fome, o que acontece ao zerar, quanto cada comida enche);
- as peças de missão ("quando o jogador comer...").

Fome, músculo, gordura, fôlego, dinheiro, propriedades, respeito, gangue, procurado e aparência funcionam assim.

## O mundo

- **Mapa novo.** Ao criar um jogo: terrenos prontos ou um mundo vazio, só com céu, luz e ciclo do dia.
- **Personagem.** CJ, qualquer personagem do San Andreas ou um criado no criador de personagem.
- **Visual.** Fiel ao San Andreas original de 2004. Leve, roda em qualquer PC.
- **HUD.** Começa igual ao do San Andreas (radar, dinheiro, texto de missão) e pode ser todo personalizado.
- **Idiomas.** Português e inglês desde o início. A comunidade traduz para outros.

## Primeiro contato

Quem abre a SAEngine pela primeira vez cai num tutorial guiado, num mapa pequeno inspirado no começo marcante do jogo: Grove Street, com a bicicleta. Ele ensina a engine inteira em miniatura.

## Do primeiro passo à 0.1

A 0.1 é a primeira fatia de teste: o tutorial da bicicleta funcionando, com o personagem andando e pedalando e a pessoa colocando objetos e montando uma regra simples. É a base. O jogo completo vem por cima dela, camada por camada.

1. Ler os arquivos do jogo (`.img`, `.dff`, `.txd`). **Concluído:** crate [`sae-rw`](crates/sae-rw).
2. Fundação de peças e capacidades.
3. Quarteirão de Grove Street.
4. CJ andando e correndo.
5. Bicicleta.
6. Catálogo de objetos no editor.
7. Peças de jogo e teste a partir de qualquer ponto.
8. Tutorial guiado.

## Depois da 0.1

Várias fases e mapas ligados, progressão, níveis e conquistas, menus e telas do jogo, salvar e continuar, mapa completo do San Andreas, modelos de fora (Blender), mods de carros e skins, galeria da comunidade, jogo independente (`.exe`), multiplayer e assistente de IA.

## Regras do projeto

- Projeto de fã, open source, sem ligação com a Rockstar Games ou a Take-Two.
- Para jogar ou criar é preciso ter o GTA San Andreas original para PC.
- Nenhum arquivo do jogo entra neste repositório, em nenhuma circunstância.
- Licença: GPL v3.
