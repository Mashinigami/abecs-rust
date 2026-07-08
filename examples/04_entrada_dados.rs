/// Exemplo 04: Entrada de Dados
///
/// Este exemplo demonstra como capturar dados digitados pelo usuário no Pinpad.
/// Útil para capturar valores, códigos, CPF, etc.
///
/// ⚠️  ATENÇÃO: Este é um comando BLOCANTE!
/// O programa aguardará até que o usuário digite os dados ou o timeout expire.
///
/// Execute com: cargo run --example 04_entrada_dados
use pinpad::{AbecsCommand, PinpadConnection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Exemplo 04: Entrada de Dados");
    println!("═══════════════════════════════════════════════════════\n");

    // Conectar ao Pinpad
    let port_name = "/dev/ttyACM0";
    println!("🔌 Conectando em {}...", port_name);
    let mut pinpad = PinpadConnection::open(port_name)?;
    println!("✅ Conectado!\n");

    // ⚠️ DESCOMENTE PARA DEBUG DETALHADO
    // pinpad.set_verbose(true);  // Mostra todos os bytes trocados

    // Abrir sessão
    let cmd = AbecsCommand::Open::new();
    pinpad.execute_typed(&cmd)?;

    // ═══════════════════════════════════════════════════════════
    // Exemplo 1: Capturar valor
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Capturando valor da transação...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("💡 Aguardando digitação no Pinpad...");
    println!("   Mensagem: DIGITE QUANTIDADE");
    println!("   Mínimo: 1 dígito");
    println!("   Máximo: 10 dígitos");
    println!("   Timeout: 60 segundos\n");

    // SPE_MSGIDX = 0x0021 (DIGITE QUANTIDADE)
    let cmd = AbecsCommand::GetData::new(
        0x0021, // Índice da mensagem pré-definida
        1,      // Mínimo de caracteres
        10,     // Máximo de caracteres
        60,     // Timeout em segundos
    );

    match pinpad.execute_typed(&cmd) {
        Ok(response) => {
            println!("✅ Valor digitado: {}\n", response.data);

            // Tentar parsear como valor monetário (centavos)
            if let Ok(valor) = response.data.parse::<u64>() {
                println!("   💰 Quantidade: {}\n", valor);
            }
        }
        Err(pinpad::AbecsError::UserCancelled) => {
            println!("❌ Operação cancelada pelo usuário (botão vermelho)\n");
            // Fechar sessão antes de sair
            let cmd = AbecsCommand::Close::new();
            let _ = pinpad.execute_typed(&cmd);
            return Ok(());
        }
        Err(e) => {
            println!("❌ Erro ou timeout: {}\n", e);
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Exemplo 2: Capturar código
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Capturando código de segurança...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("💡 Aguardando código (3 a 4 dígitos)...\n");

    // SPE_MSGIDX = 0x000C (DIGITE CÓDIGO DE SEGURANÇA)
    let cmd = AbecsCommand::GetData::new(
        0x000C, // DIGITE CÓDIGO DE SEGURANÇA
        3,      // Mínimo 3 dígitos
        4,      // Máximo 4 dígitos
        30,
    );

    match pinpad.execute_typed(&cmd) {
        Ok(response) => {
            println!("✅ Código: {}\n", response.data);
        }
        Err(pinpad::AbecsError::UserCancelled) => {
            println!("❌ Operação cancelada pelo usuário (botão vermelho)\n");
        }
        Err(e) => {
            println!("❌ Erro ou timeout: {}\n", e);
        }
    }

    // ═══════════════════════════════════════════════════════════
    // Exemplo 3: Capturar CPF
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Capturando CPF...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("💡 Digite o CPF (11 dígitos)...\n");

    // SPE_MSGIDX = 0x0007 (DIGITE O CPF)
    let cmd = AbecsCommand::GetData::new(
        0x0007, // DIGITE O CPF
        11,     // CPF tem 11 dígitos
        11, 45,
    );

    match pinpad.execute_typed(&cmd) {
        Ok(response) => {
            let cpf = &response.data;
            // Formatar CPF: XXX.XXX.XXX-XX
            if cpf.len() == 11 {
                let formatted = format!(
                    "{}.{}.{}-{}",
                    &cpf[0..3],
                    &cpf[3..6],
                    &cpf[6..9],
                    &cpf[9..11]
                );
                println!("✅ CPF: {}\n", formatted);
            } else {
                println!("✅ Dados: {}\n", cpf);
            }
        }
        Err(pinpad::AbecsError::UserCancelled) => {
            println!("❌ Operação cancelada pelo usuário (botão vermelho)\n");
        }
        Err(e) => {
            println!("❌ Erro ou timeout: {}\n", e);
        }
    }

    // ═══════════════════════════════════════════════════════════
    // 4: Capturar Evento Extendido (CEX)
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Capturar Evento Extendido...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let cmd = AbecsCommand::CheckEventExtended::any_event().with_timeout(30);
    let response = pinpad.execute_typed(&cmd)?;

    println!("Evento capturado com sucesso: {:?}", response);

    // ═══════════════════════════════════════════════════════════
    // 5: Fechar Sessão (CLO)
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let cmd = AbecsCommand::Close::new();
    pinpad.execute_typed(&cmd)?;

    println!("\n═══════════════════════════════════════════════════════");
    println!("  ✅ Exemplo concluído com sucesso!");
    println!("═══════════════════════════════════════════════════════");

    Ok(())
}
