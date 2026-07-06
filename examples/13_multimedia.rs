/// Exemplo 13: Comandos Multimídia - Carga e Exibição de Imagem
///
/// Este exemplo demonstra o uso dos comandos multimídia:
/// - Carregar um arquivo PNG (logomarca.png) no pinpad (MLI + MLR + MLE)
/// - Listar arquivos multimídia carregados (LMF)
/// - Exibir a imagem no display (DSI)
/// - Excluir o arquivo multimídia (DMF)
///
/// Execute com: cargo run --example 13_multimedia

use image::ImageFormat;
use pinpad::protocol::calculate_crc16;
use pinpad::qrcode::generate_custom_qrcode;
use pinpad::{AbecsCommand, MultimediaFileType, PinpadConnection};
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Exemplo 13: Comandos Multimídia");
    println!("═══════════════════════════════════════════════════════\n");

    // ═══════════════════════════════════════════════════════════
    // 1. Dados da imagem
    // ═══════════════════════════════════════════════════════════
    let file_data_raw = generate_custom_qrcode("https://google.com")?;

    let mut file_data = Vec::new();
    file_data_raw.write_to(&mut Cursor::new(&mut file_data), ImageFormat::Png)
        .expect("Failed to encode PNG");

    let file_size = file_data.len() as u32;
    let file_crc = calculate_crc16(&file_data);
    let file_name = "QRCODE01";

    println!("📦 Usando dados de imagem embutidos no código");
    println!("  • Tamanho: {} bytes", file_size);
    println!("  • CRC-16:  0x{:04X}", file_crc);
    println!("  • Nome:    {}\n", file_name);

    // ═══════════════════════════════════════════════════════════
    // 2. Conectar ao Pinpad
    // ═══════════════════════════════════════════════════════════
    let port_name = "/dev/ttyACM0";
    println!("🔌 Conectando em {}...", port_name);

    let mut pinpad = PinpadConnection::open(port_name)?;

    // ⚠️ DESCOMENTE PARA DEBUG DETALHADO
    // pinpad.set_verbose(true);  // Mostra todos os bytes trocados (SYN, ETB, CRC, etc)

    println!("✅ Conectado com sucesso!\n");

    // ═══════════════════════════════════════════════════════════
    // 3. Abrir sessão (OPN)
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Abrindo sessão...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let cmd = AbecsCommand::Open::new();
    pinpad.execute_typed(&cmd)?;
    println!("✅ Sessão aberta!\n");

    // ═══════════════════════════════════════════════════════════
    // 4. Iniciar carga do arquivo multimídia (MLI)
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Iniciando carga do arquivo multimídia (MLI)...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let cmd = AbecsCommand::MultimediaLoadInit::new(
        file_name,
        file_size,
        file_crc,
        MultimediaFileType::Png,
    );
    pinpad.execute_typed(&cmd)?;
    println!("✅ Carga iniciada!\n");

    // ═══════════════════════════════════════════════════════════
    // 5. Enviar dados do arquivo em blocos (MLR)
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Enviando dados do arquivo (MLR)...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let block_size = 989;
    let total_blocks = (file_data.len() + block_size - 1) / block_size;

    for (i, chunk) in file_data.chunks(block_size).enumerate() {
        print!("  📦 Bloco {}/{} ({} bytes)... ", i + 1, total_blocks, chunk.len());

        let cmd = AbecsCommand::MultimediaLoadRecord::from_single(chunk.to_vec());
        pinpad.execute_typed(&cmd)?;

        println!("✅");
    }
    println!();

    // ═══════════════════════════════════════════════════════════
    // 6. Finalizar carga do arquivo (MLE)
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Finalizando carga do arquivo (MLE)...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let cmd = AbecsCommand::MultimediaLoadEnd::new();
    pinpad.execute_typed(&cmd)?;
    println!("✅ Arquivo carregado com sucesso!\n");

    // ═══════════════════════════════════════════════════════════
    // 7. Listar arquivos multimídia (LMF)
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Listando arquivos multimídia (LMF)...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let cmd = AbecsCommand::ListMultimediaFiles::new();
    let response = pinpad.execute_typed(&cmd)?;

    if response.files.is_empty() {
        println!("  ⚠️  Nenhum arquivo multimídia encontrado.");
    } else {
        println!("  📋 Arquivos carregados:");
        for file in &response.files {
            println!("    • {}", file);
        }
    }
    println!();

    // ═══════════════════════════════════════════════════════════
    // 8. Exibir imagem no display (DSI)
    // ═══════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Exibindo imagem no display (DSI)...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let cmd = AbecsCommand::DisplayImage::new(file_name);
    pinpad.execute_typed(&cmd)?;
    println!("✅ Imagem exibida no pinpad!\n");

    println!("⏳ Aguardando 5 segundos para visualização...");
    std::thread::sleep(std::time::Duration::from_secs(5));

    // ═══════════════════════════════════════════════════════════
    // 9. Excluir arquivo multimídia (DMF)
    // ═══════════════════════════════════════════════════════════
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Excluindo arquivo multimídia (DMF)...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let cmd = AbecsCommand::DeleteMultimediaFiles::single(file_name);
    pinpad.execute_typed(&cmd)?;
    println!("✅ Arquivo '{}' excluído!\n", file_name);

    // ═══════════════════════════════════════════════════════════
    // 10. Fechar sessão (CLO)
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
