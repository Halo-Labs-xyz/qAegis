#!/usr/bin/env python3
"""
Azure Quantum IonQ Circuit Runner
Bridge script to run quantum circuits via Azure Quantum service.
"""

import sys
import json
import os
from typing import Dict, Any, Optional

try:
    from azure.quantum.cirq import AzureQuantumService
    import cirq
except ImportError:
    print(json.dumps({"error": "azure-quantum[cirq] not installed. Run: pip install 'azure-quantum[cirq]'"}))
    sys.exit(1)


def circuit_from_json(circuit_json: Dict[str, Any]) -> cirq.Circuit:
    """Convert our circuit JSON format to Cirq circuit."""
    qubits = [cirq.LineQubit(i) for i in range(circuit_json.get("qubits", 0))]
    circuit = cirq.Circuit()
    
    # Process gates by moment
    gates = circuit_json.get("gates", [])
    for moment in gates:
        for gate in moment:
            gate_type = gate.get("gate_type", "")
            gate_qubits = gate.get("qubits", [])
            
            if not gate_qubits:
                continue
                
            if gate_type == "X":
                circuit.append(cirq.X(qubits[gate_qubits[0]]))
            elif gate_type == "Y":
                circuit.append(cirq.Y(qubits[gate_qubits[0]]))
            elif gate_type == "Z":
                circuit.append(cirq.Z(qubits[gate_qubits[0]]))
            elif gate_type == "H":
                circuit.append(cirq.H(qubits[gate_qubits[0]]))
            elif gate_type == "CNOT" or gate_type == "CX":
                if len(gate_qubits) >= 2:
                    circuit.append(cirq.CNOT(qubits[gate_qubits[0]], qubits[gate_qubits[1]]))
            elif gate_type == "CZ":
                if len(gate_qubits) >= 2:
                    circuit.append(cirq.CZ(qubits[gate_qubits[0]], qubits[gate_qubits[1]]))
            elif gate_type == "Measure":
                if len(gate_qubits) >= 1:
                    key = gate.get("key", "m")
                    circuit.append(cirq.measure(qubits[gate_qubits[0]], key=key))
    
    return circuit


def run_circuit(
    resource_id: str,
    location: str,
    circuit_json: Dict[str, Any],
    repetitions: int,
    target: str = "honeywell.hqs-lt-s1-apival",
    timeout_seconds: int = 500
) -> Dict[str, Any]:
    """Run circuit on Azure Quantum Honeywell/Quantinuum target."""
    try:
        # Create Azure Quantum service
        service = AzureQuantumService(
            resource_id=resource_id,
            location=location,
            default_target=target
        )
        
        # Convert circuit
        circuit = circuit_from_json(circuit_json)
        
        # Run circuit
        result = service.run(
            program=circuit,
            repetitions=repetitions,
            target=target,
            timeout_seconds=timeout_seconds
        )
        
        # Convert results to our format
        histogram = {}
        measurements = {}
        
        # Get measurement data (Honeywell returns dict with "m_<key>" format)
        if isinstance(result, dict):
            # Honeywell API validator returns dict format
            for key, values in result.items():
                if key.startswith("m_"):
                    meas_key = key[2:]  # Remove "m_" prefix
                    measurements[meas_key] = []
                    for val in values:
                        # Convert binary string to int
                        outcome_int = int(val, 2) if isinstance(val, str) else int(val)
                        measurements[meas_key].append(outcome_int)
                        # Build histogram
                        histogram[val] = histogram.get(val, 0) + 1
        elif hasattr(result, 'histogram'):
            # Cirq Result format
            meas_key = 'b' if 'b' in result.measurements else (list(result.measurements.keys())[0] if result.measurements else None)
            if meas_key:
                hist = result.histogram(key=meas_key)
                if hist:
                    for outcome, count in hist.items():
                        if isinstance(outcome, int):
                            binary_str = format(outcome, 'b')
                        else:
                            binary_str = str(outcome)
                        histogram[binary_str] = count
        
        # Also try measurements directly from Cirq Result
        if hasattr(result, 'measurements') and result.measurements:
            for key, values in result.measurements.items():
                measurements[key] = [int(v) for v in values]
                # Build histogram from measurements
                for val in values:
                    binary_str = format(int(val), 'b')
                    histogram[binary_str] = histogram.get(binary_str, 0) + 1
        
        return {
            "success": True,
            "circuit_id": circuit_json.get("id", "unknown"),
            "repetitions": repetitions,
            "histogram": histogram,
            "measurements": measurements,
            "target": target,
        }
        
    except Exception as e:
        return {
            "success": False,
            "error": str(e),
            "circuit_id": circuit_json.get("id", "unknown"),
        }


def main():
    """Main entry point."""
    if len(sys.argv) < 2:
        print(json.dumps({"error": "Usage: azure_quantum_runner.py <circuit_json>"}))
        sys.exit(1)
    
    # Read circuit JSON from stdin or file
    if sys.argv[1] == "-":
        circuit_json = json.load(sys.stdin)
    else:
        with open(sys.argv[1], 'r') as f:
            circuit_json = json.load(f)
    
    # Get Azure Quantum credentials from environment
    resource_id = os.getenv("AZURE_QUANTUM_RESOURCE_ID", "")
    location = os.getenv("AZURE_QUANTUM_LOCATION", "")
    
    if not resource_id or not location:
        print(json.dumps({
            "error": "AZURE_QUANTUM_RESOURCE_ID and AZURE_QUANTUM_LOCATION must be set"
        }))
        sys.exit(1)
    
    # Get parameters
    repetitions = circuit_json.get("repetitions", 1000)
    target = circuit_json.get("target", "honeywell.hqs-lt-s1-apival")
    timeout = circuit_json.get("timeout_seconds", 500)
    
    # Run circuit
    result = run_circuit(
        resource_id=resource_id,
        location=location,
        circuit_json=circuit_json,
        repetitions=repetitions,
        target=target,
        timeout_seconds=timeout
    )
    
    print(json.dumps(result))


if __name__ == "__main__":
    main()
