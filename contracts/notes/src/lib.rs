#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, 
    Address, Env, String, Symbol, Vec, Map
};

/* KONTEKS DATA: 
  Menggunakan Struct untuk membungkus data catatan. 
  Menambahkan 'owner' untuk aspek keamanan dan 'timestamp' untuk aspek audit blockchain.
*/
#[contracttype]
#[derive(Clone, Debug)]
pub struct Note {
    pub id: u64,           // ID unik catatan
    pub owner: Address,    // Alamat dompet pemilik (Identity)
    pub title: String,     // Judul catatan
    pub content: String,   // Isi catatan
    pub category: Symbol,  // Kategori (misal: 'work', 'personal')
    pub updated_at: u64,   // Waktu modifikasi terakhir (Ledger Time)
}

#[contract]
pub struct NotesContract;

// Key unik untuk menyimpan seluruh Map catatan dalam storage kontrak
const NOTES_STORAGE: Symbol = symbol_short!("NOTES");

#[contractimpl]
impl NotesContract {
    
    /// KONTEKS: CREATE (Membuat Catatan)
    /// Fungsi ini memvalidasi identitas pengirim (owner) sebelum menyimpan data ke blockchain.
    pub fn create_note(env: Env, owner: Address, title: String, content: String, category: Symbol) -> u64 {
        // Keamanan: Memastikan hanya pemilik yang bisa membuat catatan atas namanya sendiri
        owner.require_auth();

        let mut notes: Map<u64, Note> = env.storage().instance().get(&NOTES_STORAGE).unwrap_or(Map::new(&env));
        
        // Menghasilkan ID unik menggunakan Pseudo-Random Number Generator bawaan Soroban
        let id = env.prng().gen::<u64>();
        
        let new_note = Note {
            id,
            owner: owner.clone(),
            title,
            content,
            category,
            updated_at: env.ledger().timestamp(), // Mengambil waktu resmi dari jaringan Stellar
        };

        notes.set(id, new_note);
        env.storage().instance().set(&NOTES_STORAGE, &notes);
        
        id // Mengembalikan ID untuk referensi di Frontend
    }

    /// KONTEKS: READ (Membaca Catatan)
    /// Fungsi ini bersifat "Self-Sovereign", hanya mengembalikan data milik Address yang meminta.
    pub fn get_my_notes(env: Env, owner: Address) -> Vec<Note> {
        let notes_map: Map<u64, Note> = env.storage().instance().get(&NOTES_STORAGE).unwrap_or(Map::new(&env));
        let mut user_notes = Vec::new(&env);

        // Iterasi melalui Map untuk memfilter catatan berdasarkan pemilik
        for (_, note) in notes_map.iter() {
            if note.owner == owner {
                user_notes.push_back(note);
            }
        }
        user_notes
    }

    /// KONTEKS: UPDATE (Memperbarui Catatan)
    /// Memastikan integritas data; hanya pemilik yang bisa mengubah isi catatan miliknya.
    pub fn update_note(env: Env, owner: Address, id: u64, title: String, content: String) -> bool {
        owner.require_auth();

        let mut notes: Map<u64, Note> = env.storage().instance().get(&NOTES_STORAGE).unwrap_or(Map::new(&env));
        
        if let Some(mut note) = notes.get(id) {
            // Validasi kepemilikan sebelum perubahan dilakukan
            if note.owner == owner {
                note.title = title;
                note.content = content;
                note.updated_at = env.ledger().timestamp(); // Update timestamp ke waktu terbaru
                
                notes.set(id, note);
                env.storage().instance().set(&NOTES_STORAGE, &notes);
                return true;
            }
        }
        false
    }

    /// KONTEKS: DELETE (Menghapus Catatan)
    /// Menghapus data secara permanen dari storage blockchain.
    pub fn delete_note(env: Env, owner: Address, id: u64) -> bool {
        owner.require_auth();

        let mut notes: Map<u64, Note> = env.storage().instance().get(&NOTES_STORAGE).unwrap_or(Map::new(&env));
        
        if let Some(note) = notes.get(id) {
            if note.owner == owner {
                notes.remove(id); // Menghapus entri dari Map
                env.storage().instance().set(&NOTES_STORAGE, &notes);
                return true;
            }
        }
        false
    }
}