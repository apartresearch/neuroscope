from os import path
import sys

from deepdecipher import Database, ModelMetadata, ServiceProvider

if len(sys.argv) < 3:
    raise RuntimeError(
        "Please specify a database file as the first argument and data path as the second argument."
    )

database_path = sys.argv[1]
data_path = sys.argv[2]

if path.isfile(database_path):
    database = Database.open(database_path)
else:
    database = Database.initialize(database_path)

model_name = "solu-6l-pile"
model = database.model(model_name)
if model is None:
    model_metadata = ModelMetadata.from_neuroscope(model_name)
    model = database.add_model(model_metadata)
assert model is not None

data_type = database.data_type("neuron2graph")
if data_type is not None and model.has_data_type(data_type):
    print("Deleting existing neuron2graph data from model.")
    model.delete_data_type(data_type)


print("Adding neuron2graph neuron graphs to model.")
model.add_neuron2graph_graphs(data_path)

service = database.service("neuron2graph")
if service is None:
    print("Adding neuron2graph service.")
    service = database.add_service("neuron2graph", ServiceProvider.neuron2graph())
