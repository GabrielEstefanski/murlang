# Murlang: A Linguagem de Programação para Murlocs

![Murloc Programador](https://media.tenor.com/g1cpnwn7ll0AAAAC/murloc-world-of-warcraft.gif)

> MRRGLRLRLRMGRRR!! *Tradução: Bem-vindo(a) à documentação oficial da Murlang!*

## Introdução

**Murlang** é a primeira linguagem de programação desenvolvida por murlocs, para murlocs... e humanos corajosos dispostos a aprender sua sabedoria ancestral. Projetada nas profundezas aquáticas da Costa de Azeroth, esta linguagem combina a elegância vocal dos murlocs com o poder da concorrência moderna e programação assíncrona.

*Aviso: Programar em Murlang com volume alto pode atrair murlocs para sua residência. Não nos responsabilizamos por invasões anfíbias.*

## Características

- 🐟 **Gramática Murloc-friendly**: Palavras-chave intuitivas para quem fala "mrgl mrgl"
- 🌊 **Multi-threading nativo**: Como um bando de murlocs atacando de múltiplas direções
- 🐙 **Processamento assíncrono**: Para quando você precisar pescar e programar ao mesmo tempo
- 🐸 **Tipagem dinâmica**: Flexível como um murloc nadando entre os recifes

## Sintaxe Básica

### Declaração de Variáveis

```
grrr nome_variavel = valor
```

*Este é o som que um murloc faz quando descobre um tesouro... ou uma variável.*

### Estruturas de Controle

#### If-Else
```
mrglif (condicao)
mrgl
    // código se verdadeiro
grl

mrglelse
mrgl
    // código se falso
grl
```

#### Loops
```
mrrg variavel = 0; variavel < 10; variavel = variavel + 1
mrgl
    // código do loop
grl
```

### Funções

```
grrrfnrrg nome_funcao (parametro1, parametro2)
mrgl
    // corpo da função
    grrr retorno = resultado
    grrrtn retorno
grl
```

*Nota: Quando um murloc define uma função, ele geralmente dança em círculos.*

## Recursos Avançados

### Multi-threading

```
mrglspawn thread_pescador
mrgl
    grrprint "Pescando comida enquanto programo!"
grl

// código principal continua executando...

mrglwait thread_pescador
```

### Async/Await

```
mrglasync grrrfnrrg buscar_tesouros (local)
mrgl
    // busca assincrona
    grrrtn tesouro
grl

// Em outro lugar:
grrr future_tesouro = mrglasync grrrblbl buscar_tesouros "costa"
mrglawait future_tesouro
```

### Pool de Threads

```
fshpoolsize 4
fshpool
mrgl
    mrglspawn tarefa1
    mrgl
        // Tarefa 1
    grl
    
    mrglspawn tarefa2
    mrgl
        // Tarefa 2
    grl
grl
```

## Exemplo Completo: Fibonacci à moda Murloc

```
grrrfnrrg fibonacci_rec (n)
mrgl
    mrglif (n <= 1)
    mrgl
        grrr retorno = n
        grrrtn retorno
    grl

    grrprint "Mrglglgl calculando Fibonacci(" + n + ")"
    
    grrr n1 = n - 1
    grrrblbl fibonacci_rec n1
    grrr fib_n1 = retorno

    grrr n2 = n - 2
    grrrblbl fibonacci_rec n2
    grrr fib_n2 = retorno

    grrr resultado = fib_n1 + fib_n2
    grrrtn resultado
grl

grrprint "MRGLMRGLMRGL! Fibonacci de 10:"
grrrblbl fibonacci_rec 10
grrprint "Resultado: " + retorno
```

## Exemplo com Threads: Pesca Paralela

```
grrrfnrrg pescar_paralelo (num_peixes)
mrgl
    mrglif (num_peixes <= 1)
    mrgl
        grrr retorno = num_peixes
        grrrtn retorno
    grl

    mrglspawn thread_pescador1
    mrgl
        grrprint "Murloc 1 pescando " + (num_peixes/2) + " peixes"
        grrrblbl pescar_paralelo (num_peixes/2)
    grl

    mrglspawn thread_pescador2
    mrgl
        grrprint "Murloc 2 pescando " + (num_peixes/2) + " peixes"
        grrrblbl pescar_paralelo (num_peixes/2)
    grl

    mrglwait thread_pescador1
    grrr peixes1 = retorno
    
    mrglwait thread_pescador2
    grrr peixes2 = retorno
    
    grrr total = peixes1 + peixes2
    grrprint "Total de peixes: " + total
    grrrtn total
grl
```

## Instalação

1. Certifique-se de estar próximo a um corpo de água (rios, lagos ou oceanos)
2. Realize o ritual de invocação murloc: "Mrglglglglgl!"
3. Execute:
   ```
   cargo install murlang
   ```
4. Opcional: Ofereça um peixe cru como agradecimento

## Executando programas

```bash
murlang meu_programa.mrgl
```

## Depuração

A depuração em Murlang envolve gritar "MRGLGLGLGL!" repetidamente até o programa funcionar. Os murlocs chamam isso de "Debugging orientado a grito".

## Contribuição

Contribuições são bem-vindas! Para contribuir:

1. Faça um fork do repositório
2. Crie uma branch: `git checkout -b minha-feature-mrgl`
3. Faça suas alterações
4. Commit com mensagens em linguagem murloc: `git commit -m "MRGLGLGL!"`
5. Push: `git push origin minha-feature-mrgl`
6. Abra um Pull Request

## Licença

Murlang é distribuída sob a licença **Mrgl Public License (MPL)**, que basicamente diz que você pode usar a linguagem desde que compartilhe seu peixe.

---

*"Aaaaughibbrgubugbugrguburgle!" - Criador da Murlang, ao compilar com sucesso pela primeira vez* 