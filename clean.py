import json


# each json object is currently separated by line

papers = []

line_cutoff = 100_000_000

with open("dataset.json", "r") as file:
    for line in file:
        if line_cutoff == 0:
            break
        paper = json.loads(line.replace("\n", " "))
        papers.append(paper)
        line_cutoff -= 1

print(len(papers))

with open("cleaned_dataset.json", "w") as file:
    json.dump(papers, file)
