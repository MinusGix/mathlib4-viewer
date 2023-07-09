pub mod decl;
pub mod js_iter;

use axum::{extract::State, routing::post, Json, Router};
use clap::Parser;
use decl::{DeclData, DeclKind};
use futures_util::StreamExt;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::PathBuf,
    sync::Arc,
};
use tower_http::services::ServeDir;

const MATHLIB4_DOCS_URL: &str =
    "https://github.com/leanprover-community/mathlib4_docs/archive/refs/heads/master.zip";

// Modified versions of the standard mathlib4 files, so that they make requests to the local server.
const DECLARATION_DATA_MODIFIED: &str = include_str!("../dist/declaration-data.js");
const SEARCH_MODIFIED: &str = include_str!("../dist/search.js");
const INSTANCES_MODIFIED: &str = include_str!("../dist/instances.js");
const IMPORTED_BY_MODIFIED: &str = include_str!("../dist/importedBy.js");

///
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "3000")]
    port: u16,
    /// The directory where data should be stored. Ex: `/home/user/.m4doc/`
    data_dir: Option<PathBuf>,
    /// The directory where mathlib docs are located. If not set, then they are downloaded and put in the data directory.
    #[arg(short, long)]
    mathlib_docs: Option<PathBuf>,
    #[arg(long, default_value = MATHLIB4_DOCS_URL)]
    mathlib_docs_url: Option<String>,

    #[arg(short, long)]
    skip_update: bool,
}

#[derive(Debug, Clone)]
struct DataState {
    addr: SocketAddr,

    data_dir: PathBuf,
    mathlib_dir: PathBuf,

    mathlib_docs_url: String,

    decl_data: Arc<DeclData>,
}
impl DataState {
    pub fn docs_dir(&self) -> PathBuf {
        self.mathlib_dir.join("docs")
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // ------

    // let had_custom_dir = args.data_dir.is_some();
    let data_dir = {
        let dir = dirs::data_dir();

        if let Some(dir) = dir {
            dir.join("m4doc/")
        } else {
            args.data_dir
                .expect("Failed to get directory where data should be stored on system")
        }
    };

    // Ensure the directory exists.
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
    }

    // ------

    let mathlib_dir = args
        .mathlib_docs
        .unwrap_or_else(|| data_dir.join("mathlib_docs/"));

    if !mathlib_dir.exists() {
        std::fs::create_dir_all(&mathlib_dir).expect("Failed to create mathlib docs directory");
    }

    // ------

    let mut state = DataState {
        addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), args.port)),
        data_dir,
        mathlib_dir,
        mathlib_docs_url: args
            .mathlib_docs_url
            .unwrap_or_else(|| MATHLIB4_DOCS_URL.to_owned()),

        decl_data: Default::default(),
    };

    // ------

    let has_mathlib_docs = std::fs::metadata(&state.docs_dir()).is_ok();

    let text = if has_mathlib_docs {
        "Mathlib docs appear to be installed. Do you wish to try updating them?"
    } else {
        "Mathlib docs do not appear to be installed. Do you wish to download them?"
    };

    let download = if args.skip_update {
        false
    } else {
        let download = inquire::Confirm::new(text)
            .with_default(!has_mathlib_docs)
            .prompt();

        if let Ok(download) = download {
            download
        } else {
            return;
        }
    };

    if download {
        // TODO: keep md5s of the original files and warn if our replacements were for a different version
        download_mathlib_docs(&state).await;
    } else if !has_mathlib_docs {
        println!("Mathlib docs are required to run this program. Exiting.");
        return;
    }

    let decl_path = state.docs_dir().join("declarations");
    let decl_data_path = decl_path.join("declaration-data.bmp");
    let decl_data = decl::load_decl_data(&decl_data_path).unwrap();

    state.decl_data = Arc::new(decl_data);

    copy_modified_files(&state);

    {
        let addr = state.addr;
        let docs_dir = state.docs_dir();
        // let app = Router::new().route("/", get(root));
        let app = Router::new()
            .route("/search_decl", post(search_decl))
            .route("/instances_for_class", post(instances_for_class))
            .route("/instances_for_type", post(instances_for_type))
            .route("/decl_name_to_link", post(decl_name_to_link))
            .route("/module_imported_by", post(module_imported_by))
            .route("/module_name_to_link", post(module_name_to_link))
            .route("/annotate_instances", post(annotate_instances))
            .route("/annotate_instances_for", post(annotate_instances_for))
            .route("/linked_imported_by", post(linked_imported_by))
            .with_state(state)
            .nest_service("/", ServeDir::new(&docs_dir));

        // TODO: only do this once it is *actually* open
        println!("Listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap()
    }
}

async fn download_mathlib_docs(state: &DataState) {
    // TODO: We may need to delete the old files first.
    // TODO: We could remark which files have changed if we're updating.

    let dest = tempfile::Builder::new().prefix("m4doc").tempdir().unwrap();

    let fname = "mathlib_docs.zip";
    let fname = dest.path().join(fname);

    println!(
        "Downloading mathlib docs to {:?} from {}",
        fname, state.mathlib_docs_url
    );

    let response = reqwest::get(&state.mathlib_docs_url)
        .await
        .expect("Failed to download mathlib docs");

    {
        let mut file = std::fs::File::create(&fname).expect("Failed to create file.");

        // TODO: progress bar
        // Stream the response body directly to a file.
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let item = item.expect("Failed to download mathlib docs");

            file.write_all(&item)
                .expect("Failed to write chunk to mathlib docs file");
        }

        file.flush().expect("Failed to flush downloaded docs");
    }

    println!("Downloaded zip. Extracting...");

    // ------

    let file = std::fs::File::open(&fname).expect("Failed to open mathlib docs file");

    let target_dir = &state.mathlib_dir;

    zip_extract::extract(file, target_dir, true).expect("Failed to extract mathlib docs");

    println!(
        "Finished downloading and extracting mathlib docs to {:?}",
        target_dir
    );
}

fn copy_modified_files(state: &DataState) {
    let dir = state.docs_dir();

    const MODIFIED: &[(&str, &str)] = &[
        ("declaration-data.js", DECLARATION_DATA_MODIFIED),
        ("search.js", SEARCH_MODIFIED),
        ("instances.js", INSTANCES_MODIFIED),
        ("importedBy.js", IMPORTED_BY_MODIFIED),
    ];

    for (filename, content) in MODIFIED {
        std::fs::write(dir.join(filename), content)
            .expect(&format!("Failed to replace with modified {}", filename));
    }
}

#[derive(Debug, Deserialize)]
struct SearchDecl {
    pub pattern: String,
    #[serde(default)]
    pub strict: bool,
    #[serde(default)]
    pub allowed_kinds: Option<Vec<DeclKind>>,
    #[serde(default)]
    pub max_results: Option<usize>,
}

const MAX_MAX_RESULTS: usize = 80;

/// Returns `Array<Decl>`
async fn search_decl(State(state): State<DataState>, Json(search): Json<SearchDecl>) -> String {
    let res = if search.strict {
        let decl = state.decl_data.search_strict(&search.pattern);

        if let Some(decl) = decl {
            vec![decl]
        } else {
            Vec::new()
        }
    } else {
        state.decl_data.search(
            &search.pattern,
            search.allowed_kinds.as_deref(),
            Some(search.max_results.unwrap_or(30).min(MAX_MAX_RESULTS)),
        )
    };

    serde_json::to_string(&res).unwrap()
}

/// Returns `Array<String>`
async fn instances_for_class(
    State(state): State<DataState>,
    Json(class_name): Json<String>,
) -> String {
    if let Some(instances) = state.decl_data.instances.get(class_name.as_str()) {
        serde_json::to_string(&instances).unwrap()
    } else {
        "[]".to_owned()
    }
}

/// Returns `Array<String>`
async fn instances_for_type(
    State(state): State<DataState>,
    Json(type_name): Json<String>,
) -> String {
    if let Some(instances) = state.decl_data.instances_for.get(type_name.as_str()) {
        serde_json::to_string(&instances).unwrap()
    } else {
        "[]".to_owned()
    }
}

/// Returns `String`
async fn decl_name_to_link(
    State(state): State<DataState>,
    Json(decl_name): Json<String>,
) -> String {
    if let Some(decl) = state.decl_data.declarations.get(decl_name.as_str()) {
        decl.doc_link.clone()
    } else {
        String::new()
    }
}

/// Returns `Array<String>`
async fn module_imported_by(
    State(state): State<DataState>,
    Json(module_name): Json<String>,
) -> String {
    if let Some(v) = state.decl_data.imported_by.get(module_name.as_str()) {
        serde_json::to_string(&v).unwrap()
    } else {
        "[]".to_owned()
    }
}

/// Returns `String`
async fn module_name_to_link(
    State(state): State<DataState>,
    Json(module_name): Json<String>,
) -> String {
    if let Some(v) = state.decl_data.modules.get(module_name.as_str()) {
        v.clone()
    } else {
        String::new()
    }
}

#[derive(Debug, Deserialize)]
struct AnnotateInstancesInfo {
    names: Vec<String>,
}

#[derive(Debug, Serialize)]
struct LinkInfo {
    name: String,
    link: String,
}

fn get_annotate_instances(
    state: &DataState,
    ann: &AnnotateInstancesInfo,
    targets: &IndexMap<Arc<str>, Vec<String>>,
) -> String {
    let mut instances = Vec::new();

    for name in ann.names.iter() {
        if let Some(insts) = targets.get(name.as_str()) {
            let insts = insts.iter().cloned().map(|inst| {
                let link = state
                    .decl_data
                    .declarations
                    .get(inst.as_str())
                    .map(|d| d.doc_link.clone())
                    .unwrap_or_default();

                LinkInfo { name: inst, link }
            });
            instances.push(insts.collect());
        } else {
            instances.push(Vec::new());
        }
    }

    serde_json::to_string(&instances).unwrap()
}

async fn annotate_instances(
    State(state): State<DataState>,
    Json(ann): Json<AnnotateInstancesInfo>,
) -> String {
    get_annotate_instances(&state, &ann, &state.decl_data.instances)
}

async fn annotate_instances_for(
    State(state): State<DataState>,
    Json(ann): Json<AnnotateInstancesInfo>,
) -> String {
    get_annotate_instances(&state, &ann, &state.decl_data.instances_for)
}

async fn linked_imported_by(
    State(state): State<DataState>,
    Json(module_name): Json<String>,
) -> String {
    if let Some(v) = state.decl_data.imported_by.get(module_name.as_str()) {
        let v = v.iter().cloned().map(|name| {
            let link = state
                .decl_data
                .modules
                .get(name.as_str())
                .map(|d| d.clone())
                .unwrap_or_default();

            LinkInfo { name, link }
        });
        serde_json::to_string(&v.collect::<Vec<_>>()).unwrap()
    } else {
        "[]".to_owned()
    }
}
