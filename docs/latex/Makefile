build-latex:
	pdflatex -shell-escape -synctex=1 -interaction=nonstopmode -file-line-error -recorder  "doc.tex"
	bibtex doc
	pdflatex -shell-escape -synctex=1 -interaction=nonstopmode -file-line-error -recorder  "doc.tex"
	pdflatex -shell-escape -synctex=1 -interaction=nonstopmode -file-line-error -recorder  "doc.tex"
