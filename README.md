
# simple-watch

Watch the output of a command.

### Usage

```
simple-watch [(-i <Interval>)|(--interval=<Interval>)] <command>
```

> [!NOTE]
> The timing is not precise. i.e. you may send an interval but there's no guarantee that it'll follow the interval exactly

### Examples

In the examples below, i got `alias sw=simple-watch`

```
sw tail -100 logs.txt
```

```
sw date
```

```
sw cargo check
```

### Installation

```
cargo install --git https://github.com/maheshbansod/simple-watch
```
