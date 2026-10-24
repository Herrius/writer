import type { LineaDiffDto } from '../../types/LineaDiffDto';

type Props = {
  lineas: LineaDiffDto[];
  etiquetaBase?: string;
  etiquetaPropuesta?: string;
};

export function DiffLineas({
  lineas,
  etiquetaBase = 'Antes',
  etiquetaPropuesta = 'Después',
}: Props) {
  return (
    <div className="diff-contenedor" tabIndex={0} aria-label="Diferencias por líneas">
      <table className="diff-tabla">
        <thead>
          <tr>
            <th scope="col" colSpan={2}>
              {etiquetaBase}
            </th>
            <th scope="col" colSpan={2}>
              {etiquetaPropuesta}
            </th>
          </tr>
        </thead>
        <tbody>
          {lineas.map((linea, indice) => {
            const clase =
              linea.tipo === 'insertada'
                ? 'diff-insertada'
                : linea.tipo === 'borrada'
                  ? 'diff-borrada'
                  : 'diff-igual';
            return (
              <tr key={`${indice}-${linea.tipo}`} className={clase}>
                <td className="diff-numero">{linea.numeroBase ?? ''}</td>
                <td className="diff-texto">
                  <span>{linea.tipo === 'insertada' ? '' : linea.texto}</span>
                </td>
                <td className="diff-numero">{linea.numeroPropuesta ?? ''}</td>
                <td className="diff-texto">
                  <span>{linea.tipo === 'borrada' ? '' : linea.texto}</span>
                </td>
              </tr>
            );
          })}
          {lineas.length === 0 && (
            <tr>
              <td colSpan={4} className="diff-vacio">
                Sin diferencias.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
