# 🎙️ Voice Mode and Auto-Accept Features

The CI tool now supports **Voice Mode** and configurable auto-accept functionality for seamless agent interactions with Claude Code.

## ✨ New Features

### 🚀 Voice Mode Command
```bash
ci agent voice AGENT_NAME
```
Instantly launches an agent with auto-accept enabled - no prompts, no delays, just immediate productivity.

### 🆓 Free Mode Flag
```bash
ci agent load AGENT_NAME --free
ci agent load AGENT_NAME -f
```
Load any agent with auto-accept enabled using the convenient `--free` flag.

### ⚙️ Configuration-Based Auto-Accept
Create `.ci-config.json` to automatically enable auto-accept for specific agents or commands:

```json
{
  "auto_accept": {
    "agents": ["CLIA", "Athena"],
    "agent_load": true,
    "global": false
  }
}
```

## 🎯 Quick Start

1. **Install/Update CI**:
   ```bash
   ./install.sh
   ```

2. **Launch an agent in voice mode**:
   ```bash
   ci agent voice CLIA
   ```

3. **Load an agent with auto-accept**:
   ```bash
   ci agent load Athena --free
   ```

## 📊 Command Comparison

| Command | Auto-Accept | Prompts | Best For |
|---------|-------------|---------|----------|
| `ci agent load AGENT` | Config-dependent | May prompt | Regular usage |
| `ci agent load AGENT --free` | ✅ Always | None | Quick development |
| `ci agent voice AGENT` | ✅ Always | None | Voice workflows |

## 🔧 Configuration Priority

1. **CLI flags** (highest) - `--free`, `voice` command
2. **Config file** - `.ci-config.json` settings  
3. **Default** (lowest) - Normal prompting

## 📁 Files Added/Modified

- **New**: `ci agent voice` command
- **Enhanced**: `ci agent load` with `--free` flag
- **New**: Auto-accept configuration system
- **Docs**: Complete voice mode documentation
- **Enhanced**: Installation script with voice mode info

## 🛡️ Security Note

Auto-accept mode allows Claude Code to execute commands without confirmation. Use responsibly and only with trusted agents in secure environments.

## 📚 Full Documentation

See `docs/VOICE_MODE.md` for complete documentation, configuration options, and best practices.

---

**Ready to experience frictionless AI collaboration! 🚀**