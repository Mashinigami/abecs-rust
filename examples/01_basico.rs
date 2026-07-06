/// Exemplo 01: Uso Básico - Abertura de Sessão e Display
///
/// Este exemplo demonstra o uso básico da biblioteca:
/// - Conectar ao Pinpad
/// - Abrir uma sessão
/// - Exibir mensagens
/// - Fechar a sessão
///
/// Execute com: cargo run --example 01_basico
use pinpad::{AbecsCommand, PinpadConnection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Exemplo 01: Uso Básico");
    println!("═══════════════════════════════════════════════════════\n");

    // ═══════════════════════════════════════════════════════════
    // 1. Listar portas disponíveis
    // ═══════════════════════════════════════════════════════════
    println!("📋 Portas seriais disponíveis:");
    let ports = PinpadConnection::list_ports()?;

    if ports.is_empty() {
        println!("  ⚠️  Nenhuma porta encontrada!");
        println!("\n💡 Dica: Conecte o Pinpad via USB");
        return Ok(());
    }

    for port in &ports {
        println!("  • {}", port);
    }

    // ═══════════════════════════════════════════════════════════
    // 2. Conectar ao Pinpad
    // ═══════════════════════════════════════════════════════════
    let port_name = "/dev/ttyACM0"; // Ajuste conforme sua porta
    println!("\n🔌 Conectando em {}...", port_name);

    let mut pinpad = PinpadConnection::open(port_name)?;

    // Opcional: ativar modo verbose para ver os bytes trocados
    // pinpad.set_verbose(true);

    println!("✅ Conectado com sucesso!\n");

    // ═══════════════════════════════════════════════════════════
    // 3. Abrir sessão (OPN)
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Abrindo sessão...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let cmd = AbecsCommand::Open::new();
    let _response = pinpad.execute_typed(&cmd)?;

    println!("✅ Sessão aberta!\n");

    // ═══════════════════════════════════════════════════════════
    // 4. Exibir mensagem (DSP)
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Exibindo mensagem no Pinpad...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let cmd = AbecsCommand::Display::new("032   BEM-VINDO!      PINPAD RUST       ");
    pinpad.execute_typed(&cmd)?;

    println!("✅ Mensagem exibida!\n");

    // Aguarda um pouco
    std::thread::sleep(std::time::Duration::from_secs(2));

    // ═══════════════════════════════════════════════════════════
    // 5. Exibir mensagem livre (DEX)
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Exibindo mensagem livre no Pinpad...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let cmd = AbecsCommand::DisplayMessage::new("12345678901234!\n  34567890123456123456789012345612345678901234561234567890123456123456789012345612345678901234561234567890123456");

    pinpad.execute_typed(&cmd)?;

    println!("✅ Mensagem livre exibida!\n");

    // Aguarda um pouco
    std::thread::sleep(std::time::Duration::from_secs(2));

    // ═══════════════════════════════════════════════════════════
    // 6. Limpar display (CLX)
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Limpando display...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let cmd = AbecsCommand::ClearDisplay::new();
    pinpad.execute_typed(&cmd)?;

    println!("✅ Display limpo!\n");

    // ═══════════════════════════════════════════════════════════
    // 6. Fechar sessão (CLO)
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Fechando sessão...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let cmd = AbecsCommand::Close::new();
    pinpad.execute_typed(&cmd)?;

    println!("✅ Sessão fechada!\n");

    println!("═══════════════════════════════════════════════════════");
    println!("  ✅ Exemplo concluído com sucesso!");
    println!("═══════════════════════════════════════════════════════");

    Ok(())
}
