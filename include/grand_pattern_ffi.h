#ifndef GRAND_PATTERN_FFI_H
#define GRAND_PATTERN_FFI_H

#include <stddef.h>
#include <stdint.h>

typedef struct GpGraph GpGraph;

// Lifecycle
GpGraph* gp_graph_new(double bpm);
void gp_graph_free(GpGraph* graph);

// Rooms
size_t gp_add_room(GpGraph* graph, double vibe);
void gp_remove_room(GpGraph* graph, size_t id);
double gp_room_vibe(const GpGraph* graph, size_t id);
double gp_room_surprise(const GpGraph* graph, size_t id);
void gp_set_room_vibe(GpGraph* graph, size_t id, double vibe);

// Edges
void gp_add_edge(GpGraph* graph, size_t from, size_t to, double weight);
size_t gp_edge_count(const GpGraph* graph);

// Operations
void gp_tick(GpGraph* graph);
void gp_diffuse(GpGraph* graph, double rate);
void gp_learn(GpGraph* graph);

// Fleet stats
double gp_total_vibe(const GpGraph* graph);
double gp_fleet_vibe(const GpGraph* graph);
double gp_fleet_surprise(const GpGraph* graph);
size_t gp_room_count(const GpGraph* graph);
uint64_t gp_tick_count(const GpGraph* graph);

// Conservation
int gp_verify_conservation(const GpGraph* graph, double tolerance);

// Topology
void gp_topology_chain(GpGraph* graph);
void gp_topology_ring(GpGraph* graph);
void gp_topology_star(GpGraph* graph);
void gp_topology_mesh(GpGraph* graph);
void gp_topology_small_world(GpGraph* graph, double probability);

// Bulk export
void gp_export_vibes(const GpGraph* graph, double* out, size_t len);
void gp_export_surprises(const GpGraph* graph, double* out, size_t len);

#endif
