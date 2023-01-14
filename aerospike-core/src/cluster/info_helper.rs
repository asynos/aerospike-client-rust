use std::str::FromStr;

use crate::errors::{ErrorKind, Result};


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServicesResponse<'a> {
    pub peers_generation: u32,
    pub port: u16,
    pub nodes: Vec<NodeResponse<'a>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeResponse<'a> {
    pub node_id: &'a str,
    pub tls_name: Option<&'a str>,
    pub endpoints: Vec<&'a str>,
}

pub fn parse_services_response<'a>(response: &'a str) -> Result<ServicesResponse<'a>> {
    // peers-generation, port, [ list of [ NodeIDs/Names, TLSName(if defined), [ List of endpoints/IPaddresses ]]]

    const COMMON : char = ',';
    const OPEN_BRACE : char = '[';
    const CLOSE_BRACE : char = ']';

    fn remove_outer_bracers<'a>(input: &'a str) -> Result<&'a str> {
        match (input.chars().next(), input.chars().nth(input.len() - 1)) {
            (Some(OPEN_BRACE), Some(CLOSE_BRACE)) => Ok(&input[1..input.len()-1]),
            _ => Result::Err(ErrorKind::BadResponse(format!("Missing outer bracers {input}")).into())
        }
    }

    fn read_generation_and_port_and_nodes<'a>(input: &'a str) -> Result<(u32, u16, Vec<NodeResponse>)> {
        
        fn read_nodes<'a>(nodes: &'a str) -> Result<Vec<NodeResponse>> {
            // [ list of [ NodeIDs/Names, TLSName(if defined), [ List of endpoints/IPaddresses ]]]
            let mut result : Vec<NodeResponse> = vec![];
    
            fn read_node<'a>(node: &'a str) -> Result<NodeResponse> {
                // [ NodeIDs/Names, TLSName(if defined), [ List of endpoints/IPaddresses ]]
    
                fn read_endpoints<'a>(endpoints: &'a str) -> Result<Vec<&'a str>> {
                    // [ List of endpoints/IPaddresses ]
    
                    let endpoints = remove_outer_bracers(endpoints)?;
                    
                    Ok(endpoints.split(COMMON).collect::<Vec<_>>())
                }
    
                let node = remove_outer_bracers(node)?;
    
                let first_common = node.find(COMMON)
                    .ok_or(ErrorKind::BadResponse("Missing section after node id".to_string()))?;
    
                let node_id = node.get(0..first_common)
                    .ok_or(ErrorKind::BadResponse("Missing node id".to_string()))?;
    
                let second_common = node[first_common+1..]
                    .find(COMMON)
                    .map(|x| x + first_common + 1)
                    .ok_or(ErrorKind::BadResponse("Missing section after tls name".to_string()))?;
    
                let tls_name = node.get(first_common+1..second_common)
                    .ok_or(ErrorKind::BadResponse("Missing tls name".to_string()))?;
                let tls_name = if tls_name.is_empty() { None } else { Some(tls_name) };
    
                let endpoints_slice = node.get(second_common+1..)
                    .ok_or(ErrorKind::BadResponse("Missing endpoints list".to_string()))?;
                
                let endpoints = read_endpoints(endpoints_slice)?;
                
                Ok(NodeResponse { node_id, tls_name, endpoints })
            }
    
            let nodes = remove_outer_bracers(nodes)?;
    
            let mut opened_bracers : i32 = 0;
            let mut first_opened_brace_pos : Option<usize> = None;
            for (pos, ch) in nodes.char_indices() {
                match ch {
                    OPEN_BRACE => {
                        if first_opened_brace_pos.is_none() {
                            first_opened_brace_pos = Some(pos);
                        }
                        opened_bracers += 1;
                        
                    },
                    CLOSE_BRACE => {
                        opened_bracers -= 1;
                        if opened_bracers < 0 {
                            return Result::Err(ErrorKind::BadResponse("Malformed nodes list".to_string()).into());
                        }
    
                        if opened_bracers == 0 {
                            let Some(opened_brace_pos) = first_opened_brace_pos else {
                                return Result::Err(ErrorKind::BadResponse("Wrong node list parser state".to_string()).into());
                            };
        
                            let node_slice = nodes.get(opened_brace_pos..pos+1)
                                .ok_or(ErrorKind::BadResponse("Invalid node slice in list".to_string()))?;
        
                            let node = read_node(node_slice)?;
                            
                            result.push(node);

                            first_opened_brace_pos = None;
                        }
                    },
                    _ => {},
                }
            }
    
            Ok(result)
        }
    
        let first_common = input
            .find(COMMON)
            .ok_or(ErrorKind::BadResponse("Missing peers generation".to_string()))?;

        let peers_generation_slice = input.get(0..first_common)
            .ok_or(ErrorKind::BadResponse("Missing peers generation".to_string()))?;

        let peers_generation = u32::from_str(peers_generation_slice)
            .map_err(|_| ErrorKind::BadResponse("Peers generation should be u32".to_string()))?;

        let second_common = input[first_common+1..]
            .find(COMMON)
            .map(|x| x + first_common + 1)
            .ok_or(ErrorKind::BadResponse("Missing port".to_string()))?;

        let port_slice = input.get(first_common+1..second_common)
            .ok_or(ErrorKind::BadResponse("Missing port".to_string()))?;

        let port = u16::from_str(port_slice)
            .map_err(|_| ErrorKind::BadResponse("TCP port should be u16".to_string()))?;
        
        let nodes = input.get(second_common+1..)
            .ok_or(ErrorKind::BadResponse("Missing node list".to_string()))?;

        let nodes = read_nodes(nodes)?;

        Ok((peers_generation, port, nodes))
    }

    let (peers_generation, port, nodes) = read_generation_and_port_and_nodes(response)?;

    Ok(ServicesResponse {
        peers_generation,
        port,
        nodes,
    })
}

mod tests {
    use crate::cluster::info_helper::{ServicesResponse, NodeResponse, parse_services_response};

    #[test]
    fn positive_cases() {
        let responses = [
            "9,3000,[[BB9040011AC4202,,[172.17.0.4]],[BB9050011AC4202,,[172.17.0.5]]]",
            "9,3000,[[BB9060011AC4202,,[74.125.239.53]],[BB9070011AC4202,,[74.125.239.54]]]",
            "10,4333,[[BB9060011AC4202,clusternode,[74.125.239.53]],[BB9070011AC4202,clusternode,[74.125.239.54]]]",
            "10,4333,[[BB9040011AC4202,clusternode,[172.17.0.4,74.125.239.53]],[BB9050011AC4202,clusternode,[172.17.0.5,74.125.239.54]]]",
        ];

        let parsed_responses = [
            ServicesResponse {
                peers_generation: 9,
                port: 3000,
                nodes: vec![
                    NodeResponse {
                        node_id: "BB9040011AC4202",
                        tls_name: None,
                        endpoints: vec![
                            "172.17.0.4",
                        ]
                    },
                    NodeResponse {
                        node_id: "BB9050011AC4202",
                        tls_name: None,
                        endpoints: vec![
                            "172.17.0.5",
                        ]
                    },
                ],
            },
            ServicesResponse {
                peers_generation: 9,
                port: 3000,
                nodes: vec![
                    NodeResponse {
                        node_id: "BB9060011AC4202",
                        tls_name: None,
                        endpoints: vec![
                            "74.125.239.53",
                        ]
                    },
                    NodeResponse {
                        node_id: "BB9070011AC4202",
                        tls_name: None,
                        endpoints: vec![
                            "74.125.239.54",
                        ]
                    },
                ],
            },
            ServicesResponse {
                peers_generation: 10,
                port: 4333,
                nodes: vec![
                    NodeResponse {
                        node_id: "BB9060011AC4202",
                        tls_name: Some("clusternode"),
                        endpoints: vec![
                            "74.125.239.53",
                        ]
                    },
                    NodeResponse {
                        node_id: "BB9070011AC4202",
                        tls_name: Some("clusternode"),
                        endpoints: vec![
                            "74.125.239.54",
                        ]
                    },
                ],
            },
            ServicesResponse {
                peers_generation: 10,
                port: 4333,
                nodes: vec![
                    NodeResponse {
                        node_id: "BB9040011AC4202",
                        tls_name: Some("clusternode"),
                        endpoints: vec![
                            "172.17.0.4",
                            "74.125.239.53",
                        ]
                    },
                    NodeResponse {
                        node_id: "BB9050011AC4202",
                        tls_name: Some("clusternode"),
                        endpoints: vec![
                            "172.17.0.5",
                            "74.125.239.54",
                        ]
                    },
                ],
            },
        ];

        for (parsed, response) in parsed_responses.iter().zip(responses.iter()) {
            assert_eq!(parsed, &parse_services_response(response).unwrap());
        }
    }
}