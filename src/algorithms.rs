use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Algorithm {
    Dijkstra,
    Johnson,
    Prim,
    RucksackPTAS,
    RucksackFPTAS,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AlgorithmType {
    Graph,
    Approximation,
}

impl Algorithm {
    fn values() -> [Algorithm; 5] {
        return [Algorithm::Dijkstra, Algorithm::Johnson, Algorithm::Prim, Algorithm::RucksackPTAS, Algorithm::RucksackFPTAS];
    }

    fn name(&self) -> &str {
        match self {
            Algorithm::Dijkstra => {
                return "Dijkstra";
            },
            Algorithm::Johnson => {
                return "Johnson";
            },
            Algorithm::Prim => {
                return "Prim";
            },
            Algorithm::RucksackPTAS => {
                return "Rucksack-PTAS";
            },
            Algorithm::RucksackFPTAS => {
                return "Rucksack-FPTAS";
            }
        }
    }

    fn is_type(&self, alg_type: AlgorithmType) -> bool {
        match alg_type {
            AlgorithmType::Graph => {
                match self {
                    Algorithm::Dijkstra => {
                        return true;
                    },
                    Algorithm::Johnson => {
                        return true;
                    },
                    Algorithm::Prim => {
                        return true;
                    },
                    _ => {
                        return false;
                    }
                }
            },
            AlgorithmType::Approximation => {
                match self {
                    Algorithm::RucksackPTAS => {
                        return true;
                    },
                    Algorithm::RucksackFPTAS => {
                        return true;
                    },
                    _ => {
                        return false;
                    }
                }
            }
        }
    }
}

/*
    request = {
        "request_type": "list" | "execute",
        "content": {
            <request-specific content>
        }
    }
*/

#[derive(Deserialize)]
struct AlgorithmRequest {
    request_type: AlgorithmRequestType,
    content: serde_json::Value
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum AlgorithmRequestType {
    List,
    Execution
}

#[derive(Deserialize)]
struct AlgorithmListRequest {
    list_type: Option<AlgorithmType>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum AlgorithmListType {
    All,
    Graph
}

#[derive(Deserialize)]
struct AlgorithmExecutionRequest {
    algorithm: Algorithm,
    data: String
}

#[derive(Serialize)]
pub enum AlgorithmExecutionResponse {
    Ok,
    Error
}


pub async fn handle_algorithm_request(request_body: String) -> Json<(AlgorithmExecutionResponse, String)> {

    let request = match serde_json::from_str::<AlgorithmRequest>(&request_body) {
        Ok(data_tl) => {
            data_tl
        },
        Err(e) => {
            let errmsg = format!("Error parsing toplevel request: {:?}", e);
            println!("{}", errmsg);
            return Json((AlgorithmExecutionResponse::Error, errmsg));
        }
    };

    match request.request_type {
        AlgorithmRequestType::List => {
            match serde_json::from_value::<AlgorithmListRequest>(request.content) {
                Ok(content_list) => {
                    return handle_list_request(content_list)
                },
                Err(e) => {
                    let errmsg = format!("Error parsing list request: {:?}", e);
                    println!("{}", errmsg);
                    return Json((AlgorithmExecutionResponse::Error, errmsg));
                }
            };
        },
        AlgorithmRequestType::Execution => {
            match serde_json::from_value::<AlgorithmExecutionRequest>(request.content) {
                Ok(content_execution) => {
                    return handle_execution_request(content_execution)
                },
                Err(e) => {
                    let errmsg = format!("Error parsing execution request: {:?}", e);
                    println!("{}", errmsg);
                    return Json((AlgorithmExecutionResponse::Error, errmsg));
                }
            };
        }
    }
}

fn handle_list_request(request: AlgorithmListRequest) -> Json<(AlgorithmExecutionResponse, String)> {
    match request.list_type {
        None => {
            let algs = serde_json::to_string(
                &Algorithm::values().iter()
                .map(|alg| alg.name()).collect::<Vec<_>>())
                .unwrap_or_else(|err| {
                    println!("Graph algorithm list jsonification failed: {:?}", err);
                    "'error': 'algorithm list json conversion failed!']".to_string()
                });
            Json((AlgorithmExecutionResponse::Ok, algs))
        },
        Some(AlgorithmType::Graph) => {
            let algs = serde_json::to_string(
                &Algorithm::values().iter()
                .filter(|alg| alg.is_type(AlgorithmType::Graph)).map(|alg| alg.name()).collect::<Vec<_>>())
                .unwrap_or_else(|err| {
                    println!("Graph algorithm list jsonification failed: {:?}", err);
                    "'error': 'algorithm list json conversion failed!']".to_string()
                });
            Json((AlgorithmExecutionResponse::Ok, algs))
        },
        Some(AlgorithmType::Approximation) => {
            let algs = serde_json::to_string(
                &Algorithm::values().iter()
                .filter(|alg| alg.is_type(AlgorithmType::Graph)).map(|alg| alg.name()).collect::<Vec<_>>())
                .unwrap_or_else(|err| {
                    println!("Graph algorithm list jsonification failed: {:?}", err);
                    "'error': 'algorithm list json conversion failed!']".to_string()
                });
            Json((AlgorithmExecutionResponse::Ok, algs))
        },
    }
}

fn handle_execution_request(request: AlgorithmExecutionRequest) -> Json<(AlgorithmExecutionResponse, String)> {
    match request.algorithm {
        Algorithm::Dijkstra => {
            return Json((AlgorithmExecutionResponse::Ok, "algorithm execution request handler Dijkstra".to_string()))
        },
        Algorithm::Johnson => {
            return Json((AlgorithmExecutionResponse::Ok, "algorithm execution request handler Johnson".to_string()))
        },
        Algorithm::Prim => {
            return Json((AlgorithmExecutionResponse::Ok, "algorithm execution request handler Prim".to_string()))
        },
        Algorithm::RucksackPTAS => {
            return Json((AlgorithmExecutionResponse::Ok, "algorithm execution request handler Rucksack-PTAS".to_string()))
        },
        Algorithm::RucksackFPTAS => {
            return Json((AlgorithmExecutionResponse::Ok, "algorithm execution request handler Rucksack-FPTAS".to_string()))
        }
    }
}