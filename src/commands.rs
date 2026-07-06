/// Comandos ABECS pré-definidos com tipos seguros
use crate::response::AbecsResponse;
use crate::serialize::{AbecsDeserialize, AbecsSerialize, AbecsTypedCommand};

// ═══════════════════════════════════════════════════════════════════════════
// Enums
// ═══════════════════════════════════════════════════════════════════════════

/// Tipo de cartão detectado pelo Pinpad
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardType {
    /// Cartão magnético (código "00")
    Magnetic,
    /// Cartão com chip ICC EMV (código "03")
    IccEmv,
    /// Cartão contactless magnético (código "05")
    CtlsMagnetic,
    /// Cartão contactless EMV (código "06")
    CtlsEmv,
    /// Tipo desconhecido ou não mapeado
    Unknown(String),
}

impl CardType {
    /// Cria um CardType a partir do código ABECS
    pub fn from_code(code: &str) -> Self {
        match code {
            "00" => CardType::Magnetic,
            "03" => CardType::IccEmv,
            "05" => CardType::CtlsMagnetic,
            "06" => CardType::CtlsEmv,
            _ => CardType::Unknown(code.to_string()),
        }
    }

    /// Retorna o código ABECS do tipo de cartão
    pub fn to_code(&self) -> String {
        match self {
            CardType::Magnetic => "00".to_string(),
            CardType::IccEmv => "03".to_string(),
            CardType::CtlsMagnetic => "05".to_string(),
            CardType::CtlsEmv => "06".to_string(),
            CardType::Unknown(code) => code.clone(),
        }
    }

    /// Retorna uma descrição legível do tipo de cartão
    pub fn description(&self) -> &str {
        match self {
            CardType::Magnetic => "Cartão Magnético",
            CardType::IccEmv => "Chip ICC EMV",
            CardType::CtlsMagnetic => "Contactless Magnético",
            CardType::CtlsEmv => "Contactless EMV",
            CardType::Unknown(_) => "Tipo Desconhecido",
        }
    }

    /// Verifica se o cartão é contactless (NFC)
    pub fn is_contactless(&self) -> bool {
        matches!(self, CardType::CtlsMagnetic | CardType::CtlsEmv)
    }

    /// Verifica se o cartão é EMV (chip ou contactless)
    pub fn is_emv(&self) -> bool {
        matches!(self, CardType::IccEmv | CardType::CtlsEmv)
    }

    /// Verifica se o cartão usa apenas tarja magnética
    pub fn is_magnetic_only(&self) -> bool {
        matches!(self, CardType::Magnetic | CardType::CtlsMagnetic)
    }
}

impl std::fmt::Display for CardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Estruturas de Dados de Cartão
// ═══════════════════════════════════════════════════════════════════════════

/// Método de pagamento identificado pelo código de serviço
///
/// ⚠️ **LIMITAÇÃO IMPORTANTE**: A detecção por service code é **não confiável** e apenas
/// uma heurística aproximada. O service code ISO/IEC 7813 não foi projetado para
/// distinguir crédito de débito.
///
/// **Fonte confiável**: Use a **mensagem NTM** do Pinpad durante o GCX, que mostra
/// o nome da aplicação selecionada (ex: "SELECIONADO: CREDITO"). Esta informação
/// vem da tabela AID configurada no Pinpad e é a fonte autorizada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentMethod {
    /// Crédito (estimativa - pode estar incorreto)
    Credit,
    /// Débito (estimativa - pode estar incorreto)
    Debit,
    /// Desconhecido ou não identificado
    Unknown,
}

impl PaymentMethod {
    /// Identifica o método de pagamento pelo código de serviço (service code)
    ///
    /// ⚠️ **IMPORTANTE**: Esta é apenas uma **heurística aproximada** e **não é confiável**.
    /// O service code ISO/IEC 7813 não foi projetado para distinguir crédito de débito.
    ///
    /// **Fonte confiável**: A informação correta vem da **mensagem NTM** do Pinpad durante
    /// o GCX, que mostra o nome da aplicação selecionada (ex: "CREDITO", "DEBITO").
    /// Essa informação vem da tabela AID configurada no Pinpad.
    ///
    /// **Service Code (ISO/IEC 7813)**:
    /// - 1º dígito: Interchange e tecnologia (1=intl chip, 2=intl chip, 5=natl chip, 6=natl chip)
    /// - 2º dígito: Procedimento de autorização (0=normal, 2=contact issuer, 4=contact except region)
    /// - 3º dígito: Serviços permitidos e restrições PIN
    ///
    /// Exemplo: Service code "201" pode ser CRÉDITO ou DÉBITO - o código não distingue.
    ///
    /// Use este método apenas como **estimativa aproximada** quando a mensagem NTM
    /// não estiver disponível.
    pub fn from_service_code(service_code: &str) -> Self {
        if service_code.len() < 3 {
            return PaymentMethod::Unknown;
        }

        // O service code ISO/IEC 7813 NÃO indica crédito vs débito de forma confiável.
        // Esta é apenas uma heurística muito aproximada baseada em padrões observados.
        let third_digit = service_code.chars().nth(2).unwrap();

        // Heurística APROXIMADA baseada no 3º dígito (serviços permitidos):
        // - 0, 2, 5, 7: Normalmente sem PIN ou PIN opcional → mais comum em crédito
        // - 1, 3, 4, 6: Normalmente requer PIN → mais comum em débito
        // Mas isso não é garantido! Use a mensagem NTM como fonte real.
        match third_digit {
            '0' | '2' | '5' | '7' => PaymentMethod::Credit,
            '1' | '3' | '4' | '6' => PaymentMethod::Debit,
            _ => PaymentMethod::Unknown,
        }
    }

    /// Retorna o nome do método de pagamento
    pub fn name(&self) -> &str {
        match self {
            PaymentMethod::Credit => "Crédito",
            PaymentMethod::Debit => "Débito",
            PaymentMethod::Unknown => "Desconhecido",
        }
    }
}

impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Dados parseados da Track 1 (Trilha 1) do cartão magnético
///
/// Formato ISO/IEC 7813 Track 1 (IATA):
/// - Início: `%` ou `B` (em BCD)
/// - PAN (Primary Account Number): até 19 dígitos
/// - Separador: `^`
/// - Nome do titular: até 26 caracteres (opcional)
/// - Separador: `^`
/// - Data de validade: YYMM (4 dígitos)
/// - Código de serviço: 3 dígitos
/// - Dados discricionários: informações adicionais do banco
/// - Fim: `?`
///
/// Exemplo em BCD: `6396649900138069D32032060000007250325F`
/// - `6396649900138069` = PAN
/// - `D` = Separador (em BCD, `D` = `^` ou `=`)
/// - `3203` = Validade (03/2032)
/// - `206` = Código de serviço
/// - `0000007250325F` = Dados discricionários
#[derive(Debug, Clone)]
pub struct Track1Data {
    /// Dados brutos da trilha em formato string
    pub raw: String,

    /// PAN (Primary Account Number) - número do cartão
    pub pan: Option<String>,

    /// Nome do titular do cartão (se disponível)
    pub cardholder_name: Option<String>,

    /// Data de validade no formato YYMM (ex: "3203" = março de 2032)
    pub expiry_date: Option<String>,

    /// Código de serviço (3 dígitos)
    pub service_code: Option<String>,

    /// Dados discricionários (informações adicionais do banco)
    pub discretionary_data: Option<String>,
}

impl Track1Data {
    /// Parseia uma string da Track 1 em formato BCD
    ///
    /// # Exemplos
    ///
    /// ```rust,no_run
    /// use pinpad::Track1Data;
    ///
    /// let track1_str = "6396649900138069D32032060000007250325F";
    /// let track1 = Track1Data::parse(track1_str);
    ///
    /// println!("PAN: {:?}", track1.pan);
    /// println!("Validade: {:?}", track1.expiry_date);
    /// ```
    pub fn parse(data: &str) -> Self {
        let raw = data.to_string();

        // Remove caractere inicial se for '%' ou 'B'
        let data = data.trim_start_matches('%').trim_start_matches('B');

        // Remove caractere final se for '?'
        let data = data.trim_end_matches('?');

        // Em BCD, o separador pode ser 'D' ou '^'
        // Tenta encontrar o primeiro separador
        let parts: Vec<&str> = if data.contains('D') {
            data.splitn(2, 'D').collect()
        } else if data.contains('^') {
            data.splitn(2, '^').collect()
        } else if data.contains('=') {
            data.splitn(2, '=').collect()
        } else {
            // Sem separador encontrado, trata tudo como PAN
            return Self {
                raw,
                pan: Some(data.to_string()),
                cardholder_name: None,
                expiry_date: None,
                service_code: None,
                discretionary_data: None,
            };
        };

        let pan = if !parts.is_empty() {
            Some(parts[0].to_string())
        } else {
            None
        };

        // Parse da parte após o PAN
        let mut cardholder_name = None;
        let mut expiry_date = None;
        let mut service_code = None;
        let mut discretionary_data = None;

        if parts.len() > 1 {
            let remainder = parts[1];

            // Verifica se tem nome do titular (se tiver outro separador '^')
            if remainder.contains('^') {
                let name_parts: Vec<&str> = remainder.splitn(2, '^').collect();
                cardholder_name = Some(name_parts[0].to_string());

                if name_parts.len() > 1 {
                    let data_part = name_parts[1];

                    // Formato esperado: YYMMCCCDDDDDD...
                    // YYMM = 4 dígitos (validade)
                    // CCC = 3 dígitos (service code)
                    // DDDDDD... = dados discricionários

                    if data_part.len() >= 4 {
                        expiry_date = Some(data_part[0..4].to_string());
                    }

                    if data_part.len() >= 7 {
                        service_code = Some(data_part[4..7].to_string());
                    }

                    if data_part.len() > 7 {
                        discretionary_data = Some(data_part[7..].to_string());
                    }
                }
            } else {
                // Sem nome, vai direto para validade
                if remainder.len() >= 4 {
                    expiry_date = Some(remainder[0..4].to_string());
                }

                if remainder.len() >= 7 {
                    service_code = Some(remainder[4..7].to_string());
                }

                if remainder.len() > 7 {
                    discretionary_data = Some(remainder[7..].to_string());
                }
            }
        }

        Self {
            raw,
            pan,
            cardholder_name,
            expiry_date,
            service_code,
            discretionary_data,
        }
    }

    /// Retorna a data de validade em formato legível (MM/YYYY)
    ///
    /// Exemplo: "3203" → "03/2032"
    pub fn expiry_date_formatted(&self) -> Option<String> {
        self.expiry_date.as_ref().and_then(|date| {
            if date.len() == 4 {
                let yy = &date[0..2];
                let mm = &date[2..4];
                // Assume que anos 00-49 são 2000-2049, 50-99 são 1950-1999
                let year = if yy.parse::<u32>().ok()? < 50 {
                    format!("20{}", yy)
                } else {
                    format!("19{}", yy)
                };
                Some(format!("{}/{}", mm, year))
            } else {
                None
            }
        })
    }

    /// Verifica se o cartão está expirado com base na data fornecida
    ///
    /// # Exemplos
    ///
    /// ```rust,no_run
    /// use pinpad::Track1Data;
    ///
    /// let track1 = Track1Data::parse("6396649900138069D3203206");
    /// // Para data atual (2025/11)
    /// assert!(!track1.is_expired(2025, 11));
    /// ```
    pub fn is_expired(&self, current_year: u32, current_month: u32) -> bool {
        self.expiry_date.as_ref().map_or(false, |date| {
            if date.len() == 4 {
                if let (Ok(yy), Ok(mm)) = (date[0..2].parse::<u32>(), date[2..4].parse::<u32>()) {
                    let year = if yy < 50 { 2000 + yy } else { 1900 + yy };

                    if year < current_year {
                        return true;
                    }
                    if year == current_year && mm < current_month {
                        return true;
                    }
                }
            }
            false
        })
    }

    /// Retorna o método de pagamento detectado pelo código de serviço
    ///
    /// ⚠️ **AVISO**: Este método retorna apenas uma **estimativa aproximada** e pode estar
    /// **incorreto**. O service code ISO não foi projetado para distinguir crédito de débito.
    ///
    /// **Fonte confiável**: A informação correta aparece na **mensagem NTM** do Pinpad durante
    /// o comando GCX (ex: "SELECIONADO: CREDITO" ou "SELECIONADO: DEBITO").
    ///
    /// **Exemplo de inconsistência**:
    /// - Service code "201" → Este método retorna `Débito`
    /// - Mensagem NTM: "CREDITO" → **Esta é a informação correta!**
    ///
    /// Use este método apenas quando a mensagem NTM não estiver disponível.
    ///
    /// # Exemplos
    ///
    /// ```rust,no_run
    /// use pinpad::Track1Data;
    ///
    /// let track1 = Track1Data::parse("6396649900138069D3203206");
    /// // ⚠️ Pode estar incorreto! Prefira usar a mensagem NTM do Pinpad
    /// println!("Estimativa: {}", track1.payment_method());
    /// ```
    pub fn payment_method(&self) -> PaymentMethod {
        self.service_code
            .as_ref()
            .map(|s| PaymentMethod::from_service_code(s))
            .unwrap_or(PaymentMethod::Unknown)
    }
}

impl std::fmt::Display for Track1Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Track1 [")?;
        if let Some(ref pan) = self.pan {
            write!(f, "PAN: {}", pan)?;
        }
        if let Some(ref name) = self.cardholder_name {
            write!(f, ", Nome: {}", name)?;
        }
        if let Some(ref exp) = self.expiry_date {
            write!(f, ", Validade: {}", exp)?;
        }
        write!(f, "]")
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Respostas
// ═══════════════════════════════════════════════════════════════════════════

/// Resposta genérica para comandos sem dados de retorno
#[derive(Debug, Clone)]
pub struct EmptyResponse;

/// Resposta do comando GetInfo
#[derive(Debug, Clone)]
pub struct GetInfoResponse {
    pub info: String,
}

/// Resposta do comando GetPin
#[derive(Debug, Clone)]
pub struct GetPinResponse {
    pub pin_block: Vec<u8>,
}

/// Resposta do comando GetData
#[derive(Debug, Clone)]
pub struct GetDataResponse {
    pub data: String,
}

/// Tipo de arquivo multimídia
#[derive(Debug, Clone, PartialEq)]
pub enum MultimediaFileType {
    Png,
    Jpg,
    Gif,
    Unknown(u8),
}

impl MultimediaFileType {
    pub fn to_byte(&self) -> u8 {
        match self {
            MultimediaFileType::Png => 0x01,
            MultimediaFileType::Jpg => 0x02,
            MultimediaFileType::Gif => 0x03,
            MultimediaFileType::Unknown(v) => *v,
        }
    }

    pub fn from_byte(b: u8) -> Self {
        match b {
            0x01 => MultimediaFileType::Png,
            0x02 => MultimediaFileType::Jpg,
            0x03 => MultimediaFileType::Gif,
            v => MultimediaFileType::Unknown(v),
        }
    }
}

impl std::fmt::Display for MultimediaFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultimediaFileType::Png => write!(f, "PNG"),
            MultimediaFileType::Jpg => write!(f, "JPG"),
            MultimediaFileType::Gif => write!(f, "GIF"),
            MultimediaFileType::Unknown(v) => write!(f, "Unknown(0x{:02X})", v),
        }
    }
}

/// Resposta do comando ListMultimediaFiles (LMF)
#[derive(Debug, Clone)]
pub struct ListMultimediaFilesResponse {
    pub files: Vec<String>,
}

/// Resposta do comando GetCard (GCX)
#[derive(Debug, Clone)]
pub struct GetCardResponse {
    /// Tipo de cartão detectado
    pub card_type: CardType,
    /// PAN do cartão (número do cartão em formato string, ex: "6396649900138069")
    pub pan: Option<String>,
    /// Trilha 1 (incompleta, pode estar mascarada)
    pub track1: Option<String>,
    /// Trilha 2 (incompleta, pode estar mascarada)
    pub track2: Option<String>,
    /// Trilha 3 (incompleta, pode estar mascarada)
    pub track3: Option<String>,
    /// Dados EMV em formato TLV (para cartões EMV)
    pub emv_data: Option<crate::emv::EmvData>,
    /// Status ICC (código de 2 dígitos)
    pub icc_status: Option<String>,
    /// Informações da tabela AID
    pub aid_table_info: Option<Vec<u8>>,
}

/// Resposta do comando Menu
#[derive(Debug, Clone)]
pub struct MenuResponse {
    /// Índice da opção selecionada (baseado em 0)
    ///
    /// Exemplo: Se o usuário seleciona a primeira opção, selected_index = 0
    pub selected_index: u8,
}

/// Resposta do comando GetTracks (GTK)
#[derive(Debug, Clone)]
pub struct GetTracksResponse {
    /// PAN (Primary Account Number) do cartão
    /// - Em claro: bytes em formato BCD (Binary Coded Decimal)
    ///   - Cada byte representa 2 dígitos hexadecimais
    ///   - Ex: [0x63, 0x96, 0x64] = "639664"
    ///   - Use `pan_as_string()` para converter automaticamente
    /// - Criptografado: bytes binários conforme método escolhido (DUKPT/MK)
    pub pan: Option<Vec<u8>>,

    /// Trilha 1 do cartão (formato ISO/IEC 7813)
    /// - Em claro: bytes em formato BCD
    ///   - Use `track1_as_string()` para converter automaticamente
    /// - Criptografada: bytes binários conforme método escolhido
    pub track1: Option<Vec<u8>>,

    /// Trilha 2 do cartão (formato ISO/IEC 7813)
    /// - Em claro: bytes em formato BCD
    ///   - Use `track2_as_string()` para converter automaticamente
    /// - Criptografada: bytes binários conforme método escolhido
    pub track2: Option<Vec<u8>>,

    /// Trilha 3 do cartão (formato ISO/IEC 7813)
    /// - Em claro: bytes em formato BCD
    ///   - Use `track3_as_string()` para converter automaticamente
    /// - Criptografada: bytes binários conforme método escolhido
    pub track3: Option<Vec<u8>>,

    /// KSN (Key Serial Number) da trilha 1 (apenas para DUKPT)
    pub track1_ksn: Option<Vec<u8>>,

    /// KSN da trilha 2 (apenas para DUKPT)
    pub track2_ksn: Option<Vec<u8>>,

    /// KSN da trilha 3 (apenas para DUKPT)
    pub track3_ksn: Option<Vec<u8>>,

    /// KSN do PAN (apenas para DUKPT)
    pub pan_ksn: Option<Vec<u8>>,

    /// KRAND criptografado (apenas para RSA)
    pub krand_enc: Option<Vec<u8>>,
}

impl GetTracksResponse {
    /// Converte bytes BCD (Binary Coded Decimal) para String
    ///
    /// No formato BCD, cada byte representa 2 dígitos hexadecimais.
    /// Ex: 0x63 0x96 0x64 → "639664"
    fn bcd_to_string(bytes: &[u8]) -> String {
        bytes
            .iter()
            .flat_map(|&b| {
                let high = (b >> 4) & 0x0F;
                let low = b & 0x0F;
                [
                    if high <= 9 {
                        char::from_digit(high as u32, 10).unwrap()
                    } else {
                        char::from(high + 0x37)
                    },
                    if low <= 9 {
                        char::from_digit(low as u32, 10).unwrap()
                    } else {
                        char::from(low + 0x37)
                    },
                ]
            })
            .collect()
    }

    /// Converte o PAN de bytes para String
    ///
    /// Tenta detectar automaticamente se os bytes estão em:
    /// - BCD (Binary Coded Decimal): cada byte = 2 dígitos
    /// - ASCII: bytes representam caracteres diretamente
    pub fn pan_as_string(&self) -> Option<String> {
        self.pan.as_ref().map(|bytes| {
            // Verifica se parece ser BCD (bytes não imprimíveis ou > 0x7F)
            let is_bcd = bytes.iter().any(|&b| b > 0x7F || (b < 0x20 && b != 0x00));

            if is_bcd {
                Self::bcd_to_string(bytes)
            } else {
                String::from_utf8_lossy(bytes).to_string()
            }
        })
    }

    /// Converte a trilha 1 de bytes para String
    pub fn track1_as_string(&self) -> Option<String> {
        self.track1.as_ref().map(|bytes| {
            let is_bcd = bytes.iter().any(|&b| b > 0x7F || (b < 0x20 && b != 0x00));

            if is_bcd {
                Self::bcd_to_string(bytes)
            } else {
                String::from_utf8_lossy(bytes).to_string()
            }
        })
    }

    /// Converte a trilha 2 de bytes para String
    pub fn track2_as_string(&self) -> Option<String> {
        self.track2.as_ref().map(|bytes| {
            let is_bcd = bytes.iter().any(|&b| b > 0x7F || (b < 0x20 && b != 0x00));

            if is_bcd {
                Self::bcd_to_string(bytes)
            } else {
                String::from_utf8_lossy(bytes).to_string()
            }
        })
    }

    /// Converte a trilha 3 de bytes para String
    pub fn track3_as_string(&self) -> Option<String> {
        self.track3.as_ref().map(|bytes| {
            let is_bcd = bytes.iter().any(|&b| b > 0x7F || (b < 0x20 && b != 0x00));

            if is_bcd {
                Self::bcd_to_string(bytes)
            } else {
                String::from_utf8_lossy(bytes).to_string()
            }
        })
    }

    /// Verifica se os dados estão criptografados (presença de KSNs)
    pub fn is_encrypted(&self) -> bool {
        self.pan_ksn.is_some()
            || self.track1_ksn.is_some()
            || self.track2_ksn.is_some()
            || self.track3_ksn.is_some()
    }

    /// Parseia a Track 1 em uma estrutura Track1Data
    ///
    /// Retorna None se a trilha não estiver disponível ou estiver criptografada.
    ///
    /// # Exemplos
    ///
    /// ```rust,no_run
    /// # use pinpad::AbecsCommand::GetTracks;
    /// # use pinpad::PinpadConnection;
    /// # let mut conn = PinpadConnection::open("/dev/ttyACM0").unwrap();
    /// let tracks_cmd = GetTracks::new_plain();
    /// let tracks_result = conn.execute_typed(&tracks_cmd).unwrap();
    ///
    /// if let Some(track1_data) = tracks_result.parse_track1() {
    ///     println!("PAN: {:?}", track1_data.pan);
    ///     println!("Validade: {:?}", track1_data.expiry_date_formatted());
    /// }
    /// ```
    pub fn parse_track1(&self) -> Option<Track1Data> {
        // Não parseia se estiver criptografado
        if self.is_encrypted() {
            return None;
        }

        self.track1_as_string().map(|s| Track1Data::parse(&s))
    }
}

/// Resposta do comando GetKey
#[derive(Debug, Clone)]
pub struct GetKeyResponse {
    pub key_check_value: Vec<u8>,
}

/// Resposta do comando GoOnChip (GOX)
#[derive(Debug, Clone)]
pub struct GoOnChipResponse {
    /// Resultado GOX - 6 dígitos indicando aprovação/PIN/status
    /// Formato: XXYYZZ onde:
    /// - XX: Status de processamento (00=OK, outros=erro)
    /// - YY: Indicação de PIN necessário (00=não, 01=sim)
    /// - ZZ: Resultado da transação
    pub gox_result: String,
    /// Dados EMV resultantes do processamento
    pub emv_data: Option<crate::emv::EmvData>,
    /// PIN Block criptografado (se PIN foi capturado)
    pub pin_block: Option<Vec<u8>>,
    /// Resultados de segurança do issuer
    pub issuer_results: Option<Vec<u8>>,
}

/// Resposta do comando FinishChip (FCX)
#[derive(Debug, Clone)]
pub struct FinishChipResponse {
    /// Resultado FCX - 3 dígitos indicando aprovação final
    /// "000" = Aprovado
    /// "001" = Negado
    pub fcx_result: String,
    /// Dados EMV finais após processamento
    pub emv_data: Option<crate::emv::EmvData>,
    /// Resultados finais de segurança do issuer
    pub issuer_results: Option<Vec<u8>>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Namespace AbecsCommand - módulo com submódulos para cada comando
// ═══════════════════════════════════════════════════════════════════════════

#[allow(non_snake_case)]
pub mod AbecsCommand {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════
    // Open - Abertura de Sessão (OPN)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct Open;

    impl Open {
        pub fn new() -> Self {
            Self
        }
    }

    impl AbecsTypedCommand for Open {
        type Response = EmptyResponse;

        fn command_id(&self) -> &str {
            "OPN"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            Vec::new()
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Close - Fechamento de Sessão (CLO)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct Close;

    impl Close {
        pub fn new() -> Self {
            Self
        }
    }

    impl AbecsTypedCommand for Close {
        type Response = EmptyResponse;

        fn command_id(&self) -> &str {
            "CLO"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            Vec::new()
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Display - Mostrar Mensagem (DSP)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct Display {
        pub message: String,
    }

    impl Display {
        pub fn new(message: impl Into<String>) -> Self {
            Self {
                message: message.into(),
            }
        }
    }

    impl AbecsTypedCommand for Display {
        type Response = EmptyResponse;

        fn command_id(&self) -> &str {
            "DSP"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            vec![self.message.serialize_abecs()]
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Display - Mostrar Mensagem Livre (DEX)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct DisplayMessage {
        pub message: String,
    }

    impl DisplayMessage {
        pub fn new(message: impl Into<String>) -> Self {
            let input = message.into();

            let lines: Vec<&str> = input.lines().collect();
            let mut processed_lines = Vec::new();
            let mut total_chars_without_newlines = 0;

            for line in lines {
                if total_chars_without_newlines + line.len() > 128 {
                    let remaining = 128 - total_chars_without_newlines;
                    if remaining > 0 {
                        let truncated_line = &line[..remaining.min(line.len())];
                        processed_lines.push(format_line_with_breaks(truncated_line));
                        total_chars_without_newlines += truncated_line.len();
                    }
                    break;
                }

                // Processa a linha adicionando quebras a cada 16 caracteres
                let formatted_line = format_line_with_breaks(line);
                total_chars_without_newlines += line.len();
                processed_lines.push(formatted_line);
            }

            // Junta as linhas com \n
            let formatted = processed_lines.join("\n");

            // Calcula o tamanho TOTAL incluindo \n
            // Contamos: caracteres originais + quebras de linha que foram adicionadas
            let num_newlines = formatted.matches('\n').count();
            let total_size = total_chars_without_newlines + num_newlines;

            // Prepend dos 3 primeiros caracteres com o tamanho (zero-padded)
            let size_str = format!("{:03}", total_size);
            let final_message = format!("{}{}", size_str, formatted);

            Self {
                message: final_message,
            }
        }
    }

    impl AbecsTypedCommand for DisplayMessage {
        type Response = EmptyResponse;

        fn command_id(&self) -> &str {
            "DEX"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            vec![self.message.serialize_abecs()]
        }
    }

    /// Função auxiliar para adicionar quebras de linha a cada 16 caracteres
    fn format_line_with_breaks(line: &str) -> String {
        let mut result = String::new();
        for (i, ch) in line.chars().enumerate() {
            if i > 0 && i % 16 == 0 {
                result.push('\n');
            }
            result.push(ch);
        }
        result
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ClearDisplay - Limpar Display (CLX)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct ClearDisplay;

    impl ClearDisplay {
        pub fn new() -> Self {
            Self
        }
    }

    impl AbecsTypedCommand for ClearDisplay {
        type Response = EmptyResponse;

        fn command_id(&self) -> &str {
            "CLX"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            Vec::new()
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GetInfo - Obter Informações (GIN)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct GetInfo {
        pub info_type: String,
    }

    impl GetInfo {
        pub fn new(info_type: impl Into<String>) -> Self {
            Self {
                info_type: info_type.into(),
            }
        }
    }

    impl AbecsTypedCommand for GetInfo {
        type Response = GetInfoResponse;

        fn command_id(&self) -> &str {
            "GIN"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            vec![self.info_type.serialize_abecs()]
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GetInfoExtended - Obter Informações Extendido (GIX)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct GetInfoExtended;

    impl GetInfoExtended {
        pub fn new() -> Self {
            Self
        }
    }

    impl AbecsTypedCommand for GetInfoExtended {
        type Response = EmptyResponse;
        fn command_id(&self) -> &str {
            "GIX"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            vec![]
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GetPin - Obter PIN (GPN)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct GetPin {
        pub message: String,
        pub min_length: u8,
        pub max_length: u8,
        pub method: String, // "0"=MK/WK:DES, "1"=MK/WK:TDES, "2"=DUKPT:DES, "3"=DUKPT:TDES
        pub key_index: String, // Índice da MK ou DUKPT (2 dígitos)
        pub working_key: String, // Working Key criptografada (32 hex chars, ignorado se DUKPT)
        pub pan: String,    // PAN do cartão
    }

    impl GetPin {
        pub fn new(
            message: impl Into<String>,
            min_length: u8,
            max_length: u8,
            method: impl Into<String>,
            key_index: impl Into<String>,
            working_key: impl Into<String>,
            pan: impl Into<String>,
        ) -> Self {
            Self {
                message: message.into(),
                min_length,
                max_length,
                method: method.into(),
                key_index: key_index.into(),
                working_key: working_key.into(),
                pan: pan.into(),
            }
        }
    }

    impl AbecsTypedCommand for GetPin {
        type Response = GetPinResponse;

        fn command_id(&self) -> &str {
            "GPN"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            // Formato do GPN segundo protocolo ABECS 2.12
            // Todos os campos devem ser concatenados em um único bloco
            let mut data = Vec::new();

            // GPN_METHOD (1 byte)
            data.extend_from_slice(self.method.as_bytes());

            // GPN_KEYIDX (2 bytes)
            data.extend_from_slice(self.key_index.as_bytes());

            // GPN_WKENC (32 bytes hex) - preenchido com zeros se DUKPT ou vazio
            let wk = if self.working_key.is_empty() || self.method == "2" || self.method == "3" {
                "00000000000000000000000000000000".to_string()
            } else {
                format!("{:0<32}", self.working_key)
            };
            data.extend_from_slice(wk.as_bytes());

            // GPN_PANLEN (2 bytes)
            let pan_len = self.pan.len().min(19);
            data.extend_from_slice(format!("{:02}", pan_len).as_bytes());

            // GPN_PAN (19 bytes, alinhado à esquerda com espaços)
            data.extend_from_slice(format!("{:<19}", self.pan).as_bytes());

            // GPN_ENTRIES (1 byte) - fixo "1"
            data.push(b'1');

            // GPN_MIN1 (2 bytes)
            data.extend_from_slice(format!("{:02}", self.min_length).as_bytes());

            // GPN_MAX1 (2 bytes)
            data.extend_from_slice(format!("{:02}", self.max_length).as_bytes());

            // GPN_MSG1 (32 bytes)
            data.extend_from_slice(format!("{:<32}", self.message).as_bytes());

            // Retorna como um único bloco
            vec![data]
        }

        fn is_blocking(&self) -> bool {
            true
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ═══════════════════════════════════════════════════════════════════════
    // GetData - Obter Dados (GCD - Get Collected Data)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct GetData {
        pub message_index: u16, // Índice da mensagem pré-definida (ver protocolo)
        pub min_length: u8,
        pub max_length: u8,
        pub timeout: u16,
    }

    impl GetData {
        pub fn new(message_index: u16, min_length: u8, max_length: u8, timeout: u16) -> Self {
            Self {
                message_index,
                min_length,
                max_length,
                timeout,
            }
        }
    }

    impl AbecsTypedCommand for GetData {
        type Response = GetDataResponse;

        fn command_id(&self) -> &str {
            "GCD"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            use crate::serialize::abecs_param;

            let mut all_params = Vec::new();

            // SPE_MSGIDX (0x000B) - 2 bytes
            let idx_bytes = [
                (self.message_index >> 8) as u8,
                (self.message_index & 0xFF) as u8,
            ];
            all_params.extend_from_slice(&abecs_param(0x000B, &idx_bytes));

            // SPE_MINDIG (0x000D) - 1 byte (opcional)
            if self.min_length > 0 {
                all_params.extend_from_slice(&abecs_param(0x000D, &[self.min_length]));
            }

            // SPE_MAXDIG (0x000E) - 1 byte (opcional)
            if self.max_length > 0 && self.max_length != 32 {
                all_params.extend_from_slice(&abecs_param(0x000E, &[self.max_length]));
            }

            // SPE_TIMEOUT (0x000C) - 1 byte (opcional)
            if self.timeout > 0 {
                all_params.extend_from_slice(&abecs_param(0x000C, &[self.timeout as u8]));
            }

            // Retorna como um único bloco
            vec![all_params]
        }

        fn is_blocking(&self) -> bool {
            true
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GetCard - Obter Cartão (GCX - Get Card Extended)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct GetCard {
        pub amount: u64,             // Valor em centavos
        pub date: String,            // Data AAMMDD (ano 2 dígitos)
        pub time: String,            // Hora HHMMSS
        pub timeout: u16,            // Timeout em segundos
        pub message: Option<String>, // Mensagem customizada (opcional)
    }

    impl GetCard {
        pub fn new(
            amount: u64,
            date: impl Into<String>,
            time: impl Into<String>,
            timeout: u16,
        ) -> Self {
            Self {
                amount,
                date: date.into(),
                time: time.into(),
                timeout,
                message: None,
            }
        }

        pub fn with_message(mut self, message: impl Into<String>) -> Self {
            self.message = Some(message.into());
            self
        }
    }

    impl AbecsTypedCommand for GetCard {
        type Response = GetCardResponse;

        fn command_id(&self) -> &str {
            "GCX"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            use crate::serialize::abecs_param;

            let mut all_params = Vec::new();

            // SPE_AMOUNT (0x0013) - valor em centavos, 12 dígitos
            let amount_str = format!("{:012}", self.amount);
            all_params.extend_from_slice(&abecs_param(0x0013, amount_str.as_bytes()));

            // SPE_TRNDATE (0x0015) - data AAMMDD
            all_params.extend_from_slice(&abecs_param(0x0015, self.date.as_bytes()));

            // SPE_TRNTIME (0x0016) - hora HHMMSS
            all_params.extend_from_slice(&abecs_param(0x0016, self.time.as_bytes()));

            // SPE_GCXOPT (0x0017) - opções: "10000" = aceita mag/ICC/CTLS e mostra valor
            all_params.extend_from_slice(&abecs_param(0x0017, b"10000"));

            // SPE_TIMEOUT (0x000C) - timeout em segundos
            all_params.extend_from_slice(&abecs_param(0x000C, &[self.timeout as u8]));

            // SPE_DSPMSG (0x001B) - mensagem customizada (opcional)
            if let Some(ref msg) = self.message {
                all_params.extend_from_slice(&abecs_param(0x001B, msg.as_bytes()));
            }

            // Retorna como um único bloco
            vec![all_params]
        }

        fn is_blocking(&self) -> bool {
            true
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GoOnChip - Processar Chip EMV (GOX)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct GoOnChip {
        pub app_type: String,             // Tipo de aplicação (ex: "04" = débito)
        pub amount: u64,                  // Valor em centavos
        pub date: String,                 // Data AAMMDD
        pub time: String,                 // Hora HHMMSS
        pub gox_options: String,          // Opções GOX (ex: "00000000")
        pub terminal_params: Vec<u8>,     // Parâmetros do terminal
        pub transaction_currency: String, // Código da moeda (ex: "0986" = BRL)
        pub emv_data: Option<crate::emv::EmvData>, // Dados EMV adicionais
    }

    impl GoOnChip {
        pub fn new(
            app_type: impl Into<String>,
            amount: u64,
            date: impl Into<String>,
            time: impl Into<String>,
            terminal_params: Vec<u8>,
        ) -> Self {
            Self {
                app_type: app_type.into(),
                amount,
                date: date.into(),
                time: time.into(),
                gox_options: "00000000".to_string(), // Padrão: sem opções especiais
                terminal_params,
                transaction_currency: "0986".to_string(), // BRL por padrão
                emv_data: None,
            }
        }

        pub fn with_options(mut self, options: impl Into<String>) -> Self {
            self.gox_options = options.into();
            self
        }

        pub fn with_currency(mut self, currency: impl Into<String>) -> Self {
            self.transaction_currency = currency.into();
            self
        }

        pub fn with_emv_data(mut self, emv_data: crate::emv::EmvData) -> Self {
            self.emv_data = Some(emv_data);
            self
        }
    }

    impl AbecsTypedCommand for GoOnChip {
        type Response = GoOnChipResponse;

        fn command_id(&self) -> &str {
            "GOX"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            use crate::serialize::abecs_param;

            let mut all_params = Vec::new();

            // SPE_APPTYPE (0x0011) - Tipo de aplicação
            all_params.extend_from_slice(&abecs_param(0x0011, self.app_type.as_bytes()));

            // SPE_AMOUNT (0x0013) - Valor em centavos, 12 dígitos
            let amount_str = format!("{:012}", self.amount);
            all_params.extend_from_slice(&abecs_param(0x0013, amount_str.as_bytes()));

            // SPE_TRNDATE (0x0015) - Data AAMMDD
            all_params.extend_from_slice(&abecs_param(0x0015, self.date.as_bytes()));

            // SPE_TRNTIME (0x0016) - Hora HHMMSS
            all_params.extend_from_slice(&abecs_param(0x0016, self.time.as_bytes()));

            // SPE_GOXOPT (0x0019) - Opções GOX
            all_params.extend_from_slice(&abecs_param(0x0019, self.gox_options.as_bytes()));

            // SPE_TRMPAR (0x001A) - Parâmetros do terminal
            all_params.extend_from_slice(&abecs_param(0x001A, &self.terminal_params));

            // SPE_TRNCURR (0x0022) - Código da moeda
            all_params
                .extend_from_slice(&abecs_param(0x0022, self.transaction_currency.as_bytes()));

            // SPE_EMVDATA (0x0023) - Dados EMV (opcional)
            if let Some(ref emv) = self.emv_data {
                let emv_bytes = emv.serialize();
                all_params.extend_from_slice(&abecs_param(0x0023, &emv_bytes));
            }

            vec![all_params]
        }

        fn is_blocking(&self) -> bool {
            true
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FinishChip - Finalizar Transação Chip (FCX)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct FinishChip {
        pub fcx_options: String,                   // Opções FCX (ex: "00")
        pub arc: String,                           // Authorization Response Code (2 bytes)
        pub emv_data: Option<crate::emv::EmvData>, // Dados EMV do issuer
    }

    impl FinishChip {
        pub fn new(arc: impl Into<String>) -> Self {
            Self {
                fcx_options: "00".to_string(), // Padrão: sem opções especiais
                arc: arc.into(),
                emv_data: None,
            }
        }

        pub fn with_options(mut self, options: impl Into<String>) -> Self {
            self.fcx_options = options.into();
            self
        }

        pub fn with_emv_data(mut self, emv_data: crate::emv::EmvData) -> Self {
            self.emv_data = Some(emv_data);
            self
        }
    }

    impl AbecsTypedCommand for FinishChip {
        type Response = FinishChipResponse;

        fn command_id(&self) -> &str {
            "FCX"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            use crate::serialize::abecs_param;

            let mut all_params = Vec::new();

            // SPE_FCXOPT (0x001C) - Opções FCX
            all_params.extend_from_slice(&abecs_param(0x001C, self.fcx_options.as_bytes()));

            // SPE_ARC (0x001D) - Authorization Response Code
            all_params.extend_from_slice(&abecs_param(0x001D, self.arc.as_bytes()));

            // SPE_EMVDATA (0x0023) - Dados EMV do issuer (opcional)
            if let Some(ref emv) = self.emv_data {
                let emv_bytes = emv.serialize();
                all_params.extend_from_slice(&abecs_param(0x0023, &emv_bytes));
            }

            vec![all_params]
        }

        fn is_blocking(&self) -> bool {
            true
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Menu - Menu de Seleção (MNU)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct Menu {
        pub title: String,
        pub options: Vec<String>,
        pub timeout: u16,
    }

    impl Menu {
        pub fn new(title: impl Into<String>, options: Vec<String>, timeout: u16) -> Self {
            Self {
                title: title.into(),
                options,
                timeout,
            }
        }
    }

    impl AbecsTypedCommand for Menu {
        type Response = MenuResponse;

        fn command_id(&self) -> &str {
            "MNU"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            use crate::serialize::abecs_param;

            let mut all_params = Vec::new();

            // SPE_TIMEOUT (0x000C) - 1 byte
            all_params.extend_from_slice(&abecs_param(0x000C, &[self.timeout as u8]));

            // SPE_MNUOPT (0x0020) - cada opção
            for option in &self.options {
                all_params.extend_from_slice(&abecs_param(0x0020, option.as_bytes()));
            }

            // SPE_DSPMSG (0x001B) - título do menu
            if !self.title.is_empty() {
                all_params.extend_from_slice(&abecs_param(0x001B, self.title.as_bytes()));
            }

            // Retorna como um único bloco
            vec![all_params]
        }

        fn is_blocking(&self) -> bool {
            true
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TableLoadInit - Iniciar Carga de Tabela (TLI)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct TableLoadInit {
        pub table_id: String,
    }

    impl TableLoadInit {
        pub fn new(table_id: impl Into<String>) -> Self {
            Self {
                table_id: table_id.into(),
            }
        }
    }

    impl AbecsTypedCommand for TableLoadInit {
        type Response = EmptyResponse;

        fn command_id(&self) -> &str {
            "TLI"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            vec![self.table_id.serialize_abecs()]
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TableLoadRecord - Carregar Registro de Tabela (TLR)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct TableLoadRecord {
        pub record_data: Vec<u8>,
    }

    impl TableLoadRecord {
        pub fn new(record_data: Vec<u8>) -> Self {
            Self { record_data }
        }
    }

    impl AbecsTypedCommand for TableLoadRecord {
        type Response = EmptyResponse;

        fn command_id(&self) -> &str {
            "TLR"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            vec![self.record_data.clone()]
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TableLoadFinish - Finalizar Carga de Tabela (TLF)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct TableLoadFinish;

    impl TableLoadFinish {
        pub fn new() -> Self {
            Self
        }
    }

    impl AbecsTypedCommand for TableLoadFinish {
        type Response = EmptyResponse;

        fn command_id(&self) -> &str {
            "TLF"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            Vec::new()
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GetKey - Obter Chave (GKY)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct GetKey {
        pub key_index: u8,
    }

    impl GetKey {
        pub fn new(key_index: u8) -> Self {
            Self { key_index }
        }
    }

    impl AbecsTypedCommand for GetKey {
        type Response = GetKeyResponse;

        fn command_id(&self) -> &str {
            "GKY"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            vec![format!("{:02}", self.key_index).serialize_abecs()]
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GetTracks - Obter Trilhas (GTK)
    // ═══════════════════════════════════════════════════════════════════════

    /// Comando GTK - Get Tracks
    ///
    /// Obtém as trilhas completas do cartão lido através de CEX ou GCX.
    /// As trilhas podem ser retornadas em claro ou criptografadas.
    ///
    /// IMPORTANTE: Este comando só pode ser usado UMA ÚNICA VEZ após CEX/GCX com sucesso.
    ///
    /// # Exemplos
    ///
    /// ```rust,no_run
    /// use pinpad::AbecsCommand::GetTracks;
    ///
    /// // Obter todas as trilhas em claro
    /// let cmd = GetTracks::new_plain();
    ///
    /// // Obter somente trilhas 1 e 2 em claro
    /// let cmd = GetTracks::new_plain()
    ///     .with_tracks(false, true, true, false); // PAN, T1, T2, T3
    ///
    /// // Obter trilhas criptografadas com DUKPT
    /// let cmd = GetTracks::new_dukpt("00", "30"); // keyidx=00, método DUKPT:TDES:DAT#1 ECB
    /// ```
    #[derive(Debug, Clone)]
    pub struct GetTracks {
        /// Quais trilhas retornar: (PAN, T1, T2, T3)
        pub tracks: Option<String>, // "1111" = todas, "0110" = só T1 e T2
        /// Método de criptografia (ex: "30" = DUKPT:TDES:DAT#1 ECB)
        pub crypt_method: Option<String>,
        /// IV para CBC (8 bytes hex)
        pub iv_cbc: Option<String>,
        /// Quantidade de dígitos em claro no início das trilhas
        pub open_digits: Option<String>,
        /// Índice da chave (MK:DAT ou DUKPT:DAT)
        pub key_index: Option<String>,
        /// Working Key criptografada pela MK (para método MK/WK)
        pub wk_enc: Option<Vec<u8>>,
        /// Módulo RSA (para método com chave aleatória)
        pub rsa_modulus: Option<Vec<u8>>,
        /// Expoente RSA (para método com chave aleatória)
        pub rsa_exponent: Option<Vec<u8>>,
    }

    impl GetTracks {
        /// Cria comando GTK para obter trilhas em claro
        pub fn new_plain() -> Self {
            Self {
                tracks: None,       // Retorna todas disponíveis
                crypt_method: None, // Sem criptografia
                iv_cbc: None,
                open_digits: None,
                key_index: None,
                wk_enc: None,
                rsa_modulus: None,
                rsa_exponent: None,
            }
        }

        /// Cria comando GTK com criptografia DUKPT
        pub fn new_dukpt(key_index: &str, method: &str) -> Self {
            Self {
                tracks: None,
                crypt_method: Some(method.to_string()),
                iv_cbc: None,
                open_digits: None,
                key_index: Some(key_index.to_string()),
                wk_enc: None,
                rsa_modulus: None,
                rsa_exponent: None,
            }
        }

        /// Define quais trilhas retornar
        pub fn with_tracks(mut self, pan: bool, t1: bool, t2: bool, t3: bool) -> Self {
            let mut tracks = String::new();
            tracks.push(if pan { '1' } else { '0' });
            tracks.push(if t1 { '1' } else { '0' });
            tracks.push(if t2 { '1' } else { '0' });
            tracks.push(if t3 { '1' } else { '0' });
            self.tracks = Some(tracks);
            self
        }

        /// Define quantidade de dígitos em claro no início das trilhas
        pub fn with_open_digits(mut self, digits: u8) -> Self {
            self.open_digits = Some(format!("{:02}", digits));
            self
        }

        /// Define IV para modo CBC
        pub fn with_iv_cbc(mut self, iv: &str) -> Self {
            self.iv_cbc = Some(iv.to_string());
            self
        }
    }

    impl AbecsTypedCommand for GetTracks {
        type Response = GetTracksResponse;

        fn command_id(&self) -> &str {
            "GTK"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            let mut blocks = Vec::new();

            // SPE_TRACKS (opcional) - [07]
            if let Some(ref tracks) = self.tracks {
                let mut block = vec![0x00, 0x07]; // Tag
                let data = tracks.as_bytes();
                block.extend_from_slice(&[(data.len() >> 8) as u8, data.len() as u8]);
                block.extend_from_slice(data);
                blocks.push(block);
            }

            // SPE_MTHDDAT (opcional) - [03]
            if let Some(ref method) = self.crypt_method {
                let mut block = vec![0x00, 0x03]; // Tag
                let data = method.as_bytes();
                block.extend_from_slice(&[(data.len() >> 8) as u8, data.len() as u8]);
                block.extend_from_slice(data);
                blocks.push(block);
            }

            // SPE_IVCBC (opcional) - [15]
            if let Some(ref iv) = self.iv_cbc {
                let mut block = vec![0x00, 0x15]; // Tag
                                                  // IV deve estar em hex string, converter para bytes
                let data: Vec<u8> = iv
                    .as_bytes()
                    .chunks(2)
                    .filter_map(|chunk| {
                        if chunk.len() == 2 {
                            let byte_str = std::str::from_utf8(chunk).ok()?;
                            u8::from_str_radix(byte_str, 16).ok()
                        } else {
                            None
                        }
                    })
                    .collect();
                let len = data.len();
                block.extend_from_slice(&[(len >> 8) as u8, len as u8]);
                block.extend_from_slice(&data);
                blocks.push(block);
            }

            // SPE_OPNDIG (opcional) - [14]
            if let Some(ref digits) = self.open_digits {
                let mut block = vec![0x00, 0x14]; // Tag
                let data = digits.as_bytes();
                block.extend_from_slice(&[(data.len() >> 8) as u8, data.len() as u8]);
                block.extend_from_slice(data);
                blocks.push(block);
            }

            // SPE_KEYIDX (mandatório se criptografia) - [09]
            if let Some(ref keyidx) = self.key_index {
                let mut block = vec![0x00, 0x09]; // Tag
                let data = keyidx.as_bytes();
                block.extend_from_slice(&[(data.len() >> 8) as u8, data.len() as u8]);
                block.extend_from_slice(data);
                blocks.push(block);
            }

            // SPE_WKENC (mandatório para MK/WK) - [0A]
            if let Some(ref wk) = self.wk_enc {
                let mut block = vec![0x00, 0x0A]; // Tag
                block.extend_from_slice(&[(wk.len() >> 8) as u8, wk.len() as u8]);
                block.extend_from_slice(wk);
                blocks.push(block);
            }

            // SPE_PBKMOD (mandatório para RSA) - [0D]
            if let Some(ref modulus) = self.rsa_modulus {
                let mut block = vec![0x00, 0x0D]; // Tag
                block.extend_from_slice(&[(modulus.len() >> 8) as u8, modulus.len() as u8]);
                block.extend_from_slice(modulus);
                blocks.push(block);
            }

            // SPE_PBKEXP (mandatório para RSA) - [0E]
            if let Some(ref exponent) = self.rsa_exponent {
                let mut block = vec![0x00, 0x0E]; // Tag
                block.extend_from_slice(&[(exponent.len() >> 8) as u8, exponent.len() as u8]);
                block.extend_from_slice(exponent);
                blocks.push(block);
            }

            blocks
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MultimediaLoadInit - Iniciar Carga de Arquivo Multimídia (MLI)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct MultimediaLoadInit {
        pub file_name: String,
        pub file_size: u32,
        pub file_crc: u16,
        pub file_type: MultimediaFileType,
    }

    impl MultimediaLoadInit {
        pub fn new(
            file_name: impl Into<String>,
            file_size: u32,
            file_crc: u16,
            file_type: MultimediaFileType,
        ) -> Self {
            Self {
                file_name: file_name.into(),
                file_size,
                file_crc,
                file_type,
            }
        }
    }

    impl AbecsTypedCommand for MultimediaLoadInit {
        type Response = EmptyResponse;

        fn command_id(&self) -> &str {
            "MLI"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            use crate::serialize::abecs_param;

            let mut all_params = Vec::new();

            // SPE_MFNAME (0x001E) - Nome do arquivo
            all_params.extend_from_slice(&abecs_param(0x001E, self.file_name.as_bytes()));

            // SPE_MFINFO (0x001F) - Informações do arquivo:
            // X4 (tamanho) + B2 (CRC) + B1 (tipo) + B3 (RUF)
            let mut mfinfo = Vec::new();
            mfinfo.extend_from_slice(&self.file_size.to_be_bytes());
            mfinfo.extend_from_slice(&self.file_crc.to_be_bytes());
            mfinfo.push(self.file_type.to_byte());
            mfinfo.extend_from_slice(&[0x00, 0x00, 0x00]); // RUF
            all_params.extend_from_slice(&abecs_param(0x001F, &mfinfo));

            vec![all_params]
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MultimediaLoadRecord - Enviar Dados de Arquivo Multimídia (MLR)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct MultimediaLoadRecord {
        pub data_blocks: Vec<Vec<u8>>,
    }

    impl MultimediaLoadRecord {
        pub fn new(data_blocks: Vec<Vec<u8>>) -> Self {
            Self { data_blocks }
        }

        pub fn from_single(data: Vec<u8>) -> Self {
            Self {
                data_blocks: vec![data],
            }
        }
    }

    impl AbecsTypedCommand for MultimediaLoadRecord {
        type Response = EmptyResponse;

        fn command_id(&self) -> &str {
            "MLR"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            use crate::serialize::abecs_param;

            let mut params_list: Vec<Vec<u8>> = Vec::new();

            // SPE_DATAIN (0x000F) - Cada bloco vira um parâmetro independente na lista
            for block in &self.data_blocks {
                // Usamos .push() para colocar o Vec<u8> gerado dentro do Vec<Vec<u8>>
                let parametro_formatado = abecs_param(0x000F, block);
                params_list.push(parametro_formatado);
            }

            params_list
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MultimediaLoadEnd - Finalizar Carga de Arquivo Multimídia (MLE)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct MultimediaLoadEnd;

    impl MultimediaLoadEnd {
        pub fn new() -> Self {
            Self
        }
    }

    impl AbecsTypedCommand for MultimediaLoadEnd {
        type Response = EmptyResponse;

        fn command_id(&self) -> &str {
            "MLE"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            Vec::new()
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ListMultimediaFiles - Listar Arquivos Multimídia (LMF)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct ListMultimediaFiles;

    impl ListMultimediaFiles {
        pub fn new() -> Self {
            Self
        }
    }

    impl AbecsTypedCommand for ListMultimediaFiles {
        type Response = ListMultimediaFilesResponse;

        fn command_id(&self) -> &str {
            "LMF"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            Vec::new()
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DeleteMultimediaFiles - Excluir Arquivos Multimídia (DMF)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct DeleteMultimediaFiles {
        pub file_names: Vec<String>,
    }

    impl DeleteMultimediaFiles {
        pub fn new(file_names: Vec<String>) -> Self {
            Self { file_names }
        }

        pub fn single(file_name: impl Into<String>) -> Self {
            Self {
                file_names: vec![file_name.into()],
            }
        }
    }

    impl AbecsTypedCommand for DeleteMultimediaFiles {
        type Response = EmptyResponse;

        fn command_id(&self) -> &str {
            "DMF"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            use crate::serialize::abecs_param;

            let mut all_params = Vec::new();

            // SPE_MFNAME (0x001E) - Nomes dos arquivos a excluir
            for name in &self.file_names {
                all_params.extend_from_slice(&abecs_param(0x001E, name.as_bytes()));
            }

            vec![all_params]
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DisplayImage - Apresentar Imagem no Display (DSI)
    // ═══════════════════════════════════════════════════════════════════════

    #[derive(Debug, Clone)]
    pub struct DisplayImage {
        pub file_name: String,
    }

    impl DisplayImage {
        pub fn new(file_name: impl Into<String>) -> Self {
            Self {
                file_name: file_name.into(),
            }
        }
    }

    impl AbecsTypedCommand for DisplayImage {
        type Response = EmptyResponse;

        fn command_id(&self) -> &str {
            "DSI"
        }

        fn serialize_params(&self) -> Vec<Vec<u8>> {
            use crate::serialize::abecs_param;

            let mut all_params = Vec::new();

            // SPE_MFNAME (0x001E) - Nome do arquivo a apresentar
            all_params.extend_from_slice(&abecs_param(0x001E, self.file_name.as_bytes()));

            vec![all_params]
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Implementações de Desserialização
// ═══════════════════════════════════════════════════════════════════════════

impl AbecsDeserialize for EmptyResponse {
    fn deserialize_abecs(_response: &AbecsResponse) -> Result<Self, String> {
        Ok(EmptyResponse)
    }
}

impl AbecsDeserialize for GetInfoResponse {
    fn deserialize_abecs(response: &AbecsResponse) -> Result<Self, String> {
        let info = response.get_string(0).unwrap_or_default();
        Ok(GetInfoResponse { info })
    }
}

impl AbecsDeserialize for GetPinResponse {
    fn deserialize_abecs(response: &AbecsResponse) -> Result<Self, String> {
        let pin_block = response
            .get_block(0)
            .ok_or("PIN block não encontrado")?
            .to_vec();

        Ok(GetPinResponse { pin_block })
    }
}

impl AbecsDeserialize for GetDataResponse {
    fn deserialize_abecs(response: &AbecsResponse) -> Result<Self, String> {
        // A resposta de GCD vem no formato TLV: PP_VALUE (0x804D)
        let block = response.get_block(0).ok_or("Bloco não encontrado")?;

        // Parse TLV: ID(2) + Len(2) + Value
        if block.len() < 4 {
            return Err("Resposta muito curta".to_string());
        }

        let param_id = ((block[0] as u16) << 8) | (block[1] as u16);
        let param_len = ((block[2] as u16) << 8) | (block[3] as u16);

        if param_id != 0x804D {
            return Err(format!("ID de parâmetro inesperado: 0x{:04X}", param_id));
        }

        if block.len() < 4 + param_len as usize {
            return Err("Dados incompletos".to_string());
        }

        let data = String::from_utf8_lossy(&block[4..4 + param_len as usize]).to_string();
        Ok(GetDataResponse { data })
    }
}

impl AbecsDeserialize for GetCardResponse {
    fn deserialize_abecs(response: &AbecsResponse) -> Result<Self, String> {
        // A resposta de GCX vem em formato TLV com múltiplos parâmetros
        let block = response.get_block(0).ok_or("Bloco não encontrado")?;

        let mut card_type_code = String::new();
        let mut pan = None;
        let mut track1 = None;
        let mut track2 = None;
        let mut track3 = None;
        let mut emv_data = None;
        let mut icc_status = None;
        let mut aid_table_info = None;

        // Parser TLV simples
        let mut pos = 0;
        while pos + 4 <= block.len() {
            let param_id = ((block[pos] as u16) << 8) | (block[pos + 1] as u16);
            let param_len = ((block[pos + 2] as u16) << 8) | (block[pos + 3] as u16);
            pos += 4;

            if pos + param_len as usize > block.len() {
                break;
            }

            let value = &block[pos..pos + param_len as usize];

            match param_id {
                0x804F => {
                    // PP_CARDTYPE
                    card_type_code = String::from_utf8_lossy(value).to_string();
                }
                0x8036 => {
                    // PP_PAN - número do cartão já vem como string ASCII
                    pan = Some(String::from_utf8_lossy(value).to_string());
                }
                0x8037 => {
                    // PP_TRK1INC
                    track1 = Some(String::from_utf8_lossy(value).to_string());
                }
                0x8038 => {
                    // PP_TRK2INC
                    track2 = Some(String::from_utf8_lossy(value).to_string());
                }
                0x8039 => {
                    // PP_TRK3INC
                    track3 = Some(String::from_utf8_lossy(value).to_string());
                }
                0x8054 => {
                    // PP_EMVDATA - Dados EMV em formato TLV
                    emv_data = crate::emv::EmvData::parse(value).ok();
                }
                0x8057 => {
                    // PP_ICCSTAT - Status ICC
                    icc_status = Some(String::from_utf8_lossy(value).to_string());
                }
                0x805A => {
                    // PP_AIDTABINFO - Informações da tabela AID
                    aid_table_info = Some(value.to_vec());
                }
                _ => {}
            }

            pos += param_len as usize;
        }

        // Converte o código string para o enum CardType
        let card_type = CardType::from_code(&card_type_code);

        Ok(GetCardResponse {
            card_type,
            pan,
            track1,
            track2,
            track3,
            emv_data,
            icc_status,
            aid_table_info,
        })
    }
}

impl AbecsDeserialize for MenuResponse {
    fn deserialize_abecs(response: &AbecsResponse) -> Result<Self, String> {
        // A resposta de MNU vem no formato TLV: PP_VALUE (0x804D)
        let block = response.get_block(0).ok_or("Bloco não encontrado")?;

        // Parse TLV: ID(2) + Len(2) + Value
        if block.len() < 4 {
            return Err("Resposta muito curta".to_string());
        }

        let param_id = ((block[0] as u16) << 8) | (block[1] as u16);
        let param_len = ((block[2] as u16) << 8) | (block[3] as u16);

        if param_id != 0x804D {
            return Err(format!("ID de parâmetro inesperado: 0x{:04X}", param_id));
        }

        if block.len() < 4 + param_len as usize {
            return Err("Dados incompletos".to_string());
        }

        let index_str = String::from_utf8_lossy(&block[4..4 + param_len as usize]);
        let option_number = index_str
            .trim()
            .parse::<u8>()
            .map_err(|e| format!("Erro ao parsear índice: {}", e))?;

        // O Pinpad retorna o número da opção (1, 2, 3...), mas precisamos do índice do array (0, 1, 2...)
        let selected_index = if option_number > 0 {
            option_number - 1
        } else {
            return Err("Número de opção inválido (deve ser >= 1)".to_string());
        };

        Ok(MenuResponse { selected_index })
    }
}

impl AbecsDeserialize for GetKeyResponse {
    fn deserialize_abecs(response: &AbecsResponse) -> Result<Self, String> {
        let key_check_value = response
            .get_block(0)
            .ok_or("Key check value não encontrado")?
            .to_vec();

        Ok(GetKeyResponse { key_check_value })
    }
}

impl AbecsDeserialize for GoOnChipResponse {
    fn deserialize_abecs(response: &AbecsResponse) -> Result<Self, String> {
        // A resposta de GOX vem em formato TLV com múltiplos parâmetros
        let block = response.get_block(0).ok_or("Bloco não encontrado")?;

        let mut gox_result = String::new();
        let mut emv_data = None;
        let mut pin_block = None;
        let mut issuer_results = None;

        // Parser TLV
        let mut pos = 0;
        while pos + 4 <= block.len() {
            let param_id = ((block[pos] as u16) << 8) | (block[pos + 1] as u16);
            let param_len = ((block[pos + 2] as u16) << 8) | (block[pos + 3] as u16);
            pos += 4;

            if pos + param_len as usize > block.len() {
                break;
            }

            let value = &block[pos..pos + param_len as usize];

            match param_id {
                0x8050 => {
                    // PP_GOXRES - Resultado GOX (6 dígitos)
                    gox_result = String::from_utf8_lossy(value).to_string();
                }
                0x8054 => {
                    // PP_EMVDATA - Dados EMV em formato TLV
                    emv_data = crate::emv::EmvData::parse(value).ok();
                }
                0x8055 => {
                    // PP_PINBLK - PIN Block criptografado
                    pin_block = Some(value.to_vec());
                }
                0x8056 => {
                    // PP_ISRESULTS - Resultados do issuer
                    issuer_results = Some(value.to_vec());
                }
                _ => {}
            }

            pos += param_len as usize;
        }

        Ok(GoOnChipResponse {
            gox_result,
            emv_data,
            pin_block,
            issuer_results,
        })
    }
}

impl AbecsDeserialize for FinishChipResponse {
    fn deserialize_abecs(response: &AbecsResponse) -> Result<Self, String> {
        // A resposta de FCX vem em formato TLV com múltiplos parâmetros
        let block = response.get_block(0).ok_or("Bloco não encontrado")?;

        let mut fcx_result = String::new();
        let mut emv_data = None;
        let mut issuer_results = None;

        // Parser TLV
        let mut pos = 0;
        while pos + 4 <= block.len() {
            let param_id = ((block[pos] as u16) << 8) | (block[pos + 1] as u16);
            let param_len = ((block[pos + 2] as u16) << 8) | (block[pos + 3] as u16);
            pos += 4;

            if pos + param_len as usize > block.len() {
                break;
            }

            let value = &block[pos..pos + param_len as usize];

            match param_id {
                0x8051 => {
                    // PP_FCXRES - Resultado FCX (3 dígitos)
                    fcx_result = String::from_utf8_lossy(value).to_string();
                }
                0x8054 => {
                    // PP_EMVDATA - Dados EMV em formato TLV
                    emv_data = crate::emv::EmvData::parse(value).ok();
                }
                0x8056 => {
                    // PP_ISRESULTS - Resultados do issuer
                    issuer_results = Some(value.to_vec());
                }
                _ => {}
            }

            pos += param_len as usize;
        }

        Ok(FinishChipResponse {
            fcx_result,
            emv_data,
            issuer_results,
        })
    }
}

impl AbecsDeserialize for ListMultimediaFilesResponse {
    fn deserialize_abecs(response: &AbecsResponse) -> Result<Self, String> {
        let mut files = Vec::new();

        // A resposta pode não ter blocos se não houver arquivos
        if let Some(block) = response.get_block(0) {
            // Parser TLV: múltiplos PP_MFNAME (0x805E)
            let mut pos = 0;
            while pos + 4 <= block.len() {
                let param_id = ((block[pos] as u16) << 8) | (block[pos + 1] as u16);
                let param_len = ((block[pos + 2] as u16) << 8) | (block[pos + 3] as u16);
                pos += 4;

                if pos + param_len as usize > block.len() {
                    break;
                }

                let value = &block[pos..pos + param_len as usize];

                if param_id == 0x805E {
                    // PP_MFNAME - Nome do arquivo
                    let name = String::from_utf8_lossy(value).trim().to_string();
                    files.push(name);
                }

                pos += param_len as usize;
            }
        }

        Ok(ListMultimediaFilesResponse { files })
    }
}

impl AbecsDeserialize for GetTracksResponse {
    fn deserialize_abecs(response: &AbecsResponse) -> Result<Self, String> {
        // A resposta de GTK vem em formato TLV com múltiplos parâmetros
        let block = response.get_block(0).ok_or("Bloco não encontrado")?;

        let mut pan = None;
        let mut track1 = None;
        let mut track2 = None;
        let mut track3 = None;
        let mut track1_ksn = None;
        let mut track2_ksn = None;
        let mut track3_ksn = None;
        let mut pan_ksn = None;
        let mut krand_enc = None;

        // Parser TLV
        let mut pos = 0;
        while pos + 4 <= block.len() {
            let param_id = ((block[pos] as u16) << 8) | (block[pos + 1] as u16);
            let param_len = ((block[pos + 2] as u16) << 8) | (block[pos + 3] as u16);
            pos += 4;

            if pos + param_len as usize > block.len() {
                break;
            }

            let value = &block[pos..pos + param_len as usize];

            match param_id {
                0x804A => {
                    // PP_ENCPAN - PAN (em claro ou criptografado)
                    pan = Some(value.to_vec());
                }
                0x8045 => {
                    // PP_TRACK1 - Trilha 1
                    track1 = Some(value.to_vec());
                }
                0x8042 => {
                    // PP_TRACK2 - Trilha 2
                    track2 = Some(value.to_vec());
                }
                0x8043 => {
                    // PP_TRACK3 - Trilha 3
                    track3 = Some(value.to_vec());
                }
                0x804B => {
                    // PP_TRK1KSN - KSN da trilha 1
                    track1_ksn = Some(value.to_vec());
                }
                0x804C => {
                    // PP_TRK2KSN - KSN da trilha 2
                    track2_ksn = Some(value.to_vec());
                }
                0x804D => {
                    // PP_TRK3KSN - KSN da trilha 3
                    track3_ksn = Some(value.to_vec());
                }
                0x8049 => {
                    // PP_ENCPANKSN - KSN do PAN
                    pan_ksn = Some(value.to_vec());
                }
                0x8048 => {
                    // PP_ENCKRAND - KRAND criptografado
                    krand_enc = Some(value.to_vec());
                }
                _ => {}
            }

            pos += param_len as usize;
        }

        Ok(GetTracksResponse {
            pan,
            track1,
            track2,
            track3,
            track1_ksn,
            track2_ksn,
            track3_ksn,
            pan_ksn,
            krand_enc,
        })
    }
}
